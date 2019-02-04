//! type Application = Request -> (Response -> IO ResponseReceived) -> IO ResponseReceived

extern crate futures;
extern crate http;

use futures::{Future, IntoFuture, Stream};
use http::{Request, Response, StatusCode};
use std::io;
use std::convert::AsRef;

#[derive(Clone,Debug)]
pub struct ResponseReceived;

pub trait Application {
  type Output: IntoFuture<Item=ResponseReceived, Error=io::Error>;
  type Body: Stream<Item=Self::BodyStream, Error=()>;
  type BodyStream: AsRef<[u8]>;

  fn run<F>(&mut self, req: Request<Self::Body>, respond: F) -> Self::Output
    where F: Fn(Response<Self::Body>) -> Self::Output;
}

pub trait Middleware {
  type Input: Application;
  type Output: Application;

  fn apply(i: Self::Input) -> Self::Output;
}

#[cfg(test)]
mod tests {
  use super::*;
  use futures::future::{self, FutureResult};
  use futures::stream::{Empty, empty};

  pub struct MyApp {
    s: String,
    counter: u8,
  }

  pub struct Server<App> {
    app: App,
    counter: u8,
  }

  pub struct LoggingMiddleware<A> {
    app: A,
  }

  impl<App> Server<App> {
    pub fn new(app: App) -> Self {
      Server { app, counter: 0 }
    }
  }

  impl<Bs: AsRef<[u8]>, App: Application<Output=FutureResult<ResponseReceived, io::Error>, Body=Empty<Bs, ()>, BodyStream=Bs>> Server<App> {
    pub fn run(&mut self) {
      for i in 1..3 {
        std::thread::sleep_ms(500);

        let req = Request::builder()
          .method("GET")
          .uri("/")
          .header("Host", "lolcatho.st")
          .body(empty()).unwrap();
        let mut received = self.app.run(req, |response| {
          println!("callback called");
          future::ok(ResponseReceived)
        });

        received.poll().unwrap();
      }
      panic!();
    }
  }

  impl Application for MyApp {
    type Output = FutureResult<ResponseReceived, io::Error>;
    type Body = Empty<Vec<u8>, ()>;
    type BodyStream = Vec<u8>;

    fn run<F>(&mut self, req: Request<Self::Body>, respond: F) -> Self::Output
      where F: Fn(Response<Self::Body>) -> Self::Output {
       println!("{}|app got request: {:?}", self.counter, req);
       self.counter += 1;
       let rep = Response::builder()
         .status(StatusCode::OK)
         .body(empty()).unwrap();
       respond(rep)
    }
  }

  impl<A:Application> Middleware for LoggingMiddleware<A> {
    type Input = A;
    type Output = Self;
    fn apply(a: A) -> LoggingMiddleware<A> {
      LoggingMiddleware { app: a }
    }
  }

  impl<A: Application> Application for LoggingMiddleware<A> {
    type Output = <A as Application>::Output;
    type Body = <A as Application>::Body;
    type BodyStream = <A as Application>::BodyStream;

    fn run<F>(&mut self, req: Request<Self::Body>, respond: F) -> Self::Output
      where F: Fn(Response<Self::Body>) -> Self::Output {
      println!("logging");
      self.app.run(req, respond)
    }
  }


  #[test]
  fn test() {
    let mut server = Server::new(
      LoggingMiddleware::apply(MyApp { s: "app".to_string(), counter: 0 })
    );
    server.run();
  }
}
