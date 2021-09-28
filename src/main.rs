pub mod http {
	#[derive(Copy, Clone, PartialOrd, PartialEq)]
	pub enum Method {
		Get,
		Post,
		Put,
		Delete,
	}

	pub struct HttpRequest {
		pub method: Method,
		pub path: &'static str,
		pub body: &'static str,
	}

	impl HttpRequest {
		fn method(&self) -> Method {
			self.method.clone()
		}
		fn path(&self) -> &'static str {
			self.path
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
		req.body.parse().unwrap()
	}
}

impl FromRequest for f64 {
	fn from_request(req: &HttpRequest) -> Self {
		req.body.parse().unwrap()
	}
}

impl FromRequest for String {
	fn from_request(req: &HttpRequest) -> Self {
		"string".to_string()
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

trait HandlerFactory<P, R> {
	fn exec(&self, req: &HttpRequest) -> HttpResponse;
	fn handler(&self) -> Box<dyn Fn(&HttpRequest) -> HttpResponse + '_> {
		Box::new(move |r| self.exec(r))
	}
}


impl<T, R> HandlerFactory<(),R> for T
	where
		R: Responder,
		T: Fn() -> R,
{
	fn exec(&self, _req: &HttpRequest) -> HttpResponse {
		(self)().response_to()
	}
}

impl<T, R, A> HandlerFactory<(A,),R> for T
	where
		R: Responder,
		T: Fn(A) -> R,
		A: FromRequest,
{
	fn exec(&self, req: &HttpRequest) -> HttpResponse {
		let a = A::from_request(&req);
		(self)(a).response_to()
	}
}

impl<T, R, A, B> HandlerFactory<(A, B),R> for T
	where
		R: Responder,
		T: Fn(A, B) -> R,
		A: FromRequest,
		B: FromRequest,
{
	fn exec(&self, req: &HttpRequest) -> HttpResponse {
		let a = A::from_request(&req);
		let b = B::from_request(&req);
		(*self)(a, b).response_to()
	}
}

impl<T, R, A, B, C> HandlerFactory<(A, B, C),R> for T
	where
		R: Responder,
		T: Fn(A, B, C) -> R,
		A: FromRequest,
		B: FromRequest,
		C: FromRequest,
{
	fn exec(&self, req: &HttpRequest) -> HttpResponse {
		let a = A::from_request(&req);
		let b = B::from_request(&req);
		let c = C::from_request(&req);
		(*self)(a, b, c).response_to()
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

	pub fn add<H, P, R>(
		&mut self,
		method: http::Method,
		path: &'static str,
		handler: &'static H,
	) -> &Self
		where
			H: HandlerFactory<P, R> + 'static,
			P: 'static,
			R: Responder,
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
			if r.method == req.method && r.path == req.path {
				find = true;
				let h = &r.handler;
				//let rsp = h.exec(req).body;
				let rsp = h(&req);
				println!("response to {}: {}", r.path, rsp.body);
			}
		}
		if !find {
			println!("response to {}: handler not find",req.path);
		}
	}
}

// no parameter, no return value
fn test0() {

}

// no parameter, return i32
fn test01() -> i32 {
	1
}

// i32 parameter, return i32
fn test1(i: i32) -> String {
	i.to_string()
}

// String, f64 parameters, return i32
fn test2(msg: String, f: f64) -> String {
	msg.clone()
}

// i32, String parameters, f64 return
fn test2a(i: i32, msg: String) -> f64 {
	(i as f64)
}

// String, f64, i32 parameters, f64 return
fn test3(msg: String, f: f64, i: i32) -> f64 {
	f
}

fn main() {
	let mut d = Dispatcher::new();
	d.add(http::Method::Get, "/hello", &test0);
	d.add(http::Method::Get, "/hi", &test01);
	d.add(http::Method::Get, "/login", &test1);
	d.add(http::Method::Post, "/users/1", &test2);
	d.add(http::Method::Put, "/logout", &test2a);
	d.add(http::Method::Delete, "/users", &test3);

	let mut req0 = HttpRequest {
		method: Method::Get,
		path: "/hello",
		body: "",
	};
	d.dispatch(&req0);

	let mut req01 = HttpRequest {
		method: Method::Get,
		path: "/hi",
		body: "",
	};
	d.dispatch(&req01);

	let mut req1 = HttpRequest {
		method: Method::Get,
		path: "/login",
		body: "123",
	};
	d.dispatch(&req1);

	let mut req2 = HttpRequest {
		method: Method::Post,
		path: "/users/1",
		body: "456",
	};
	d.dispatch(&req2);

	let mut req2a = HttpRequest {
		method: Method::Put,
		path: "/logout",
		body: "456",
	};
	d.dispatch(&req2a);

	let mut req3 = HttpRequest {
		method: Method::Delete,
		path: "/users",
		body: "789",
	};
	d.dispatch(&req3);

	let mut req4 = HttpRequest {
		method: Method::Delete,
		path: "/netexist",
		body: "789",
	};
	d.dispatch(&req4);
}
