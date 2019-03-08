use super::context::Args;
use super::work;
use conveyor::futures::prelude::*;
use conveyor::{Result, Station};
use conveyor_work::package::Package;
use std::pin::Pin;
use std::sync::Arc;

pub struct WorkBoxWrapper<V: 'static + Send> {
    inner: work::WorkBox<V>,
}

impl<V: 'static + Send> WorkBoxWrapper<V> {
    pub fn new(inner: work::WorkBox<V>) -> WorkBoxWrapper<V> {
        WorkBoxWrapper { inner }
    }
}

impl<V: 'static + Send> Station for WorkBoxWrapper<V> {
    type Input = V;
    type Output = Vec<work::WorkOutput<V>>;
    type Future = Pin<Box<Future<Output = Result<Self::Output>> + Send>>;

    fn execute(&self, input: Self::Input) -> Self::Future {
        self.inner.execute(input)
    }
}

pub struct WorkArcWrapper<V: 'static + Send> {
    inner: Arc<work::WorkBox<V>>,
}

impl<V: 'static + Send> WorkArcWrapper<V> {
    pub fn new(inner: Arc<work::WorkBox<V>>) -> WorkArcWrapper<V> {
        WorkArcWrapper { inner }
    }
}

impl<V: 'static + Send> Station for WorkArcWrapper<V> {
    type Input = V;
    type Output = Vec<work::WorkOutput<V>>;
    type Future = Pin<Box<Future<Output = Result<Self::Output>> + Send>>;

    fn execute(&self, input: Self::Input) -> Self::Future {
        self.inner.execute(input)
    }
}

pub struct StationFnCtx2<F, I, O, Ctx: 'static> {
    inner: F,
    ctx: std::sync::Arc<Ctx>,
    _i: std::marker::PhantomData<I>,
    _o: std::marker::PhantomData<O>,
}

impl<'a, F, I, O, Ctx: 'static, U> Station for StationFnCtx2<F, I, O, Ctx>
where
    F: (Fn(I, std::sync::Arc<Ctx>) -> U) + Send + Sync + std::marker::Unpin,
    U: Future<Output = Result<O>> + Send + 'static,
{
    type Future = U;
    type Input = I;
    type Output = O;

    fn execute(&self, input: Self::Input) -> Self::Future {
        (self.inner)(input, self.ctx.clone())
    }
}

pub fn station_fn_ctx2<F, I, O, Ctx: 'static, U>(f: F, ctx: Arc<Ctx>) -> StationFnCtx2<F, I, O, Ctx>
where
    F: (Fn(I, Arc<Ctx>) -> U) + Send + Sync + std::marker::Unpin,
    U: Future<Output = Result<O>> + Send + 'static,
{
    StationFnCtx2 {
        inner: f,
        ctx: ctx,
        _i: std::marker::PhantomData,
        _o: std::marker::PhantomData,
    }
}

use regex;

lazy_static! {
    static ref TEST: regex::Regex = regex::Regex::new(r"\$\{(\w+)\}").unwrap();
}

pub fn is_interpolated(input: &str) -> bool {
    TEST.is_match(input)
}

pub fn interpolate(input: &str, args: &Args) -> String {
    let mut output = input.to_string();
    for cap in TEST.captures_iter(input) {
        if args.contains_key(&cap[1]) {
            output = output.replace(
                &format!("${{{}}}", &cap[1]),
                args.get(&cap[1]).unwrap().as_str().unwrap(),
            );
        }
    }
    output
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn interpolated() {
        assert!(!is_interpolated("Hello, World"));
        assert!(is_interpolated("Hello, ${World}"));

        assert_eq!(
            interpolate(
                "Hello, ${toffe}!",
                &args! {
                    "toffe" => "World"
                },
            ),
            "Hello, World!"
        );

        assert_eq!(
            interpolate(
                "${ost} Hello, ${toffe}!",
                &args! {
                    "toffe" => "World",
                    "ost" => "tapas"
                },
            ),
            "tapas Hello, World!"
        );
    }

}
