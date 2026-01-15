// Application Layer
pub trait UseCase {
    type Input;
    type Output;
    
    fn execute(&self, input: Self::Input) -> Self::Output;
}
