pub trait Executor {
    type Input;

    fn set_input(&mut self, input: Self::Input);

    fn run(&mut self);

    fn run_fn(&mut self, fn_name: &str);
}
