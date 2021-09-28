pub mod http {
	#[derive(Copy, Clone, PartialOrd, PartialEq)]
	pub enum Method {
		Get,
		Post,
		Put,
		Delete,
	}

	pub struct Request {
		pub method: Method,
		pub path: &'static str,
		pub body: &'static str,
	}

	impl Request {
		fn method(&self) -> Method {
			self.method.clone()
		}
		fn path(&self) -> &'static str {
			self.path
		}
	}
	pub struct Response {}
}

use self::http::{Method, Request, Response};

trait FromRequest {
	fn from_request(req: &Request) -> Self;
}

impl FromRequest for i32 {
	fn from_request(req: &Request) -> Self {
		req.body.parse().unwrap()
	}
}

impl FromRequest for f64 {
	fn from_request(req: &Request) -> Self {
		req.body.parse().unwrap()
	}
}

impl FromRequest for String {
	fn from_request(req: &Request) -> Self {
		"string".to_string()
	}
}

trait RequestHandler {
	fn exec(&self, req: &Request) -> Response;
}

trait HandlerFactory<T> {
	fn exec(&self, req: &Request) -> Response;
	fn handler(&self) -> Box<dyn Fn(&Request) -> Response + '_> {
		Box::new(move |r| self.exec(r))
	}
}

// 为无参数的handler实现接口
impl<T> HandlerFactory<()> for T
where
	T: Fn() -> Response,
{
	fn exec(&self, _req: &Request) -> Response {
		(self)()
	}
}

// 为一个参数的handler实现接口
impl<T, A> HandlerFactory<(A,)> for T
where
	T: Fn(A) -> Response,
	A: FromRequest,
{
	fn exec(&self, req: &Request) -> Response {
		let a = A::from_request(&req);
		(self)(a)
	}
}

// 为两个参数的handler实现接口
impl<T, A, B> HandlerFactory<(A, B)> for T
where
	T: Fn(A, B) -> Response,
	A: FromRequest,
	B: FromRequest,
{
	fn exec(&self, req: &Request) -> Response {
		let a = A::from_request(&req);
		let b = B::from_request(&req);
		(*self)(a, b)
	}
}

// 为三个参数的handler实现接口
impl<T, A, B, C> HandlerFactory<(A, B, C)> for T
where
	T: Fn(A, B, C) -> Response,
	A: FromRequest,
	B: FromRequest,
	C: FromRequest,
{
	fn exec(&self, req: &Request) -> Response {
		let a = A::from_request(&req);
		let b = B::from_request(&req);
		let c = C::from_request(&req);
		(*self)(a, b, c)
	}
}

type BoxedHandler = Box<dyn Fn(&Request) -> Response>;

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

	pub fn add<H, P>(
		&mut self,
		method: http::Method,
		path: &'static str,
		handler: &'static H,
	) -> &Self
	where
		H: HandlerFactory<P> + 'static,
		P: 'static,
	{
		//println!("handle route {:?}", method);
		self.handlers.push(RouteItem {
			path,
			method,
			handler: handler.handler(),
		});
		self
	}

	fn dispatch(&self, req: &Request) {
		for r in self.handlers.iter() {
			if r.method == req.method && r.path == req.path {
				let h = &r.handler;
				h(&req);
			}
		}
	}
}

fn test0() -> Response {
	println!("test");
	Response {}
}

fn test1(i: i32) -> Response {
	println!("i={}", i);
	Response {}
}

fn test2(msg: String, v: f64) -> Response {
	println!("msg={} v={}", msg, v);
	Response {}
}
fn test2a(i: i32, msg: String) -> Response {
	println!("i={} msg={}", i, msg);
	Response {}
}

fn test3(msg: String, v: f64, i: i32) -> Response {
	println!("msg={} v={} i={}", msg, v, i);
	Response {}
}

fn main() {
	let mut d = Dispatcher::new();
	d.add(http::Method::Get, "/hello", &test0);
	d.add(http::Method::Get, "/login", &test1);
	d.add(http::Method::Post, "/users/1", &test2);
	d.add(http::Method::Put, "/logout", &test2a);
	d.add(http::Method::Delete, "/users", &test3);

	let mut req0 = Request {
		method: Method::Get,
		path: "/hello",
		body: "",
	};
	d.dispatch(&req0);

	let mut req1 = Request {
		method: Method::Get,
		path: "/login",
		body: "123",
	};
	d.dispatch(&req1);

	let mut req2 = Request {
		method: Method::Put,
		path: "/logout",
		body: "456",
	};
	d.dispatch(&req2);

	let mut req3 = Request {
		method: Method::Delete,
		path: "/users",
		body: "789",
	};
	d.dispatch(&req3);
}
