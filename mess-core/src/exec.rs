use std::{any::Any, error::Error};

use crate::artifact::Artifact;

pub trait Executor {
    type Input: Artifact;
    type Error: Error;

    fn set_input(&mut self, input: Self::Input);

    fn run(&mut self) -> Result<(), Self::Error>;

    fn run_fn(&mut self, fn_name: &str) -> Result<(), Self::Error>;
}
