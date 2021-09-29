use std::future::Future;
use futures::executor;
pub mod http {
	#[derive(Copy, Clone, PartialOrd, PartialEq, Debug)]
	pub enum Method {
		Get,
		Post,
		Put,
		Delete,
	}

	pub struct HttpRequest {
		method: Method,
		path: &'static str,
		body: &'static str,
	}

	impl HttpRequest {
		pub fn new(method: Method, path: &'static str, body: &'static str) -> Self{
			HttpRequest{method, path, body}
		}

		pub fn method(&self) -> Method {
			self.method.clone()
		}

		pub fn path(&self) -> &'static str {
			self.path
		}

		pub fn body(&self) -> &'static str {
			self.body
		}
	}

	pub struct HttpResponse {
		pub body: String,
	}
}

use self::http::{HttpRequest, HttpResponse, Method};

trait FromRequest {
	fn from_request(req: &HttpRequest) -> Self;
}

impl FromRequest for i32 {
	fn from_request(req: &HttpRequest) -> Self {
		req.body().parse().unwrap()
	}
}

impl FromRequest for f64 {
	fn from_request(req: &HttpRequest) -> Self {
		req.body().parse().unwrap()
	}
}

impl FromRequest for String {
	fn from_request(req: &HttpRequest) -> Self {
		req.body().to_string()
	}
}

trait Responder {
	fn response_to(&self) -> HttpResponse;
}

impl Responder for () {
	fn response_to(&self) -> HttpResponse {
		HttpResponse {
			body: "()".to_owned(),
		}
	}
}

impl Responder for String {
	fn response_to(&self) -> HttpResponse {
		HttpResponse { body: self.clone() }
	}
}

impl Responder for i32 {
	fn response_to(&self) -> HttpResponse {
		HttpResponse {
			body: self.to_string(),
		}
	}
}

impl Responder for f64 {
	fn response_to(&self) -> HttpResponse {
		HttpResponse {
			body: self.to_string(),
		}
	}
}

trait HandlerFactory<P, R, O> {
	fn exec(&self, req: &HttpRequest) -> HttpResponse;
	fn handler(&self) -> Box<dyn Fn(&HttpRequest) -> HttpResponse + '_> {
		Box::new(move |r| self.exec(r))
	}
}

impl<T, R, O> HandlerFactory<(),R,O> for T
	where
		T: Fn() -> R,
		R: Future<Output = O>,
		O: Responder,
{
	fn exec(&self, _req: &HttpRequest) -> HttpResponse {
		executor::block_on((self)()).response_to()

	}
}

impl<T, R, O, A> HandlerFactory<(A,),R,O> for T
	where
		T: Fn(A) -> R,
		R: Future<Output = O>,
		O: Responder,
		A: FromRequest,
{
	fn exec(&self, req: &HttpRequest) -> HttpResponse {
		let a = A::from_request(&req);
		executor::block_on((self)(a)).response_to()
	}
}

impl<T, R, O, A, B> HandlerFactory<(A, B),R,O> for T
	where
		T: Fn(A, B) -> R,
		R: Future<Output = O>,
		O: Responder,
		A: FromRequest,
		B: FromRequest,
{
	fn exec(&self, req: &HttpRequest) -> HttpResponse {
		let a = A::from_request(&req);
		let b = B::from_request(&req);
		executor::block_on((self)(a,b)).response_to()
	}
}

impl<T, R,O, A, B, C> HandlerFactory<(A, B, C),R,O> for T
	where
		T: Fn(A, B, C) -> R,
		R: Future<Output = O>,
		O: Responder,
		A: FromRequest,
		B: FromRequest,
		C: FromRequest,
{
	fn exec(&self, req: &HttpRequest) -> HttpResponse {
		let a = A::from_request(&req);
		let b = B::from_request(&req);
		let c = C::from_request(&req);
		executor::block_on((self)(a,b,c)).response_to()
	}
}

type BoxedHandler = Box<dyn Fn(&HttpRequest) -> HttpResponse>;

struct RouteItem {
	path: &'static str,
	method: http::Method,
	handler: BoxedHandler,
}

struct Dispatcher {
	handlers: Vec<RouteItem>,
}

impl Dispatcher {
	pub fn new() -> Self {
		Dispatcher {
			handlers: Vec::new(),
		}
	}

	pub fn add<H, P, R, O>(
		&mut self,
		method: http::Method,
		path: &'static str,
		handler: &'static H,
	) -> &Self
		where
			H: HandlerFactory<P, R, O> + 'static,
			P: 'static,
			R: Future<Output = O>,
			O: Responder,
	{
		//println!("handle route {:?}", method);
		self.handlers.push(RouteItem {
			path,
			method,
			handler: handler.handler(),
		});
		self
	}

	fn dispatch(&self, req: &HttpRequest) {
		let mut find = false;
		for r in self.handlers.iter() {
			if r.method == req.method() && r.path == req.path() {
				find = true;
				let h = &r.handler;
				let rsp = h(&req);
				println!("response to {:?} {}: {}", r.method, r.path, rsp.body);
			}
		}
		if !find {
			println!("response to {:?} {}: handler not find",req.method(), req.path());
		}
	}
}

// no parameter, no return value
async fn hello() {

}

// no parameter, return i32
async fn hi() -> i32 {
	2021
}

// i32 parameter, return i32
async fn login(name: String, password: String) -> String {
	format!("{}-{}", name, password)
}

// String, f64 parameters, return i32
async fn new_user(msg: String, f: f64) -> String {
	msg.clone()
}

// i32, String parameters, f64 return
async fn logout(i: i32, msg: String) -> f64 {
	i as f64
}

// String, f64, i32 parameters, f64 return
async fn list_user(msg: String, f: f64, i: i32) -> f64 {
	f
}

fn main() {

	let mut d = Dispatcher::new();

	d.add(http::Method::Get, "/hello", &hello);
	d.add(http::Method::Get, "/hi", &hi);
	d.add(http::Method::Get, "/login", &login);
	d.add(http::Method::Post, "/users/1", &new_user);
	d.add(http::Method::Put, "/logout", &logout);
	d.add(http::Method::Delete, "/users", &list_user);

	d.dispatch(&HttpRequest::new(Method::Get, "/hello",""));
	d.dispatch(&HttpRequest::new(Method::Get, "/hi", ""));
	d.dispatch(&HttpRequest::new(Method::Get,"/login","123"));
	d.dispatch(&HttpRequest::new(Method::Post,"/users/1","456"));
	d.dispatch(&HttpRequest::new(Method::Put, "/logout","321"));
	d.dispatch(&HttpRequest::new(Method::Delete,"/users","789"));
	d.dispatch(&HttpRequest::new(Method::Delete, "/not-exist","789"));

}
