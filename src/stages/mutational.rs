use crate::corpus::testcase::Testcase;
use crate::engines::Evaluator;
use crate::inputs::Input;
use crate::mutators::Mutator;
use crate::stages::{HasEvaluator, Stage};
use crate::utils::{HasRand, Rand};
use crate::AflError;

use std::cell::RefCell;
use std::rc::Rc;

// TODO create HasMutatorsVec trait

pub trait MutationalStage<'a, I>: Stage<'a, I> + HasRand
where
    I: Input,
{
    fn mutators(&self) -> &Vec<Box<dyn Mutator<I, R = Self::R>>>;

    fn mutators_mut(&mut self) -> &mut Vec<Box<dyn Mutator<I, R = Self::R>>>;

    fn add_mutator(&mut self, mutator: Box<dyn Mutator<I, R = Self::R>>) {
        self.mutators_mut().push(mutator);
    }

    fn iterations(&mut self) -> usize {
        1 + self.rand_below(128) as usize
    }

    fn perform_mutational(&mut self, entry: Rc<RefCell<Testcase<I>>>) -> Result<(), AflError> {
        let num = self.iterations();
        let mut input = entry.borrow_mut().load_input()?.clone();

        for i in 0..num {
            for m in self.mutators_mut() {
                m.mutate(&mut input, i as i32)?;
            }

            let interesting = self.eval_mut().evaluate_input(&mut input, entry.clone())?;

            for m in self.mutators_mut() {
                m.post_exec(interesting, i as i32)?;
            }

            input = entry.borrow_mut().load_input()?.clone();
        }
        Ok(())
    }
}

pub struct DefaultMutationalStage<'a, I, R, E>
where
    I: Input,
    R: Rand,
    E: Evaluator<I>,
{
    rand: Rc<RefCell<R>>,
    eval: &'a mut E,
    mutators: Vec<Box<dyn Mutator<I, R = R>>>,
}

impl<'a, I, R, E> HasRand for DefaultMutationalStage<'a, I, R, E>
where
    I: Input,
    R: Rand,
    E: Evaluator<I>,
{
    type R = R;

    fn rand(&self) -> &Rc<RefCell<R>> {
        &self.rand
    }

}

impl<'a, I, R, E> HasEvaluator<I> for DefaultMutationalStage<'a, I, R, E>
where
    I: Input,
    R: Rand,
    E: Evaluator<I>,
{
    type E = E;

    fn eval(&self) -> &Self::E {
        self.eval
    }

    fn eval_mut(&mut self) -> &mut Self::E {
        self.eval
    }
}

impl<'a, I, R, E> MutationalStage<'a, I> for DefaultMutationalStage<'a, I, R, E>
where
    I: Input,
    R: Rand,
    E: Evaluator<I>,
{
    fn mutators(&self) -> &Vec<Box<dyn Mutator<I, R = Self::R>>> {
        &self.mutators
    }

    fn mutators_mut(&mut self) -> &mut Vec<Box<dyn Mutator<I, R = Self::R>>> {
        &mut self.mutators
    }
}

impl<'a, I, R, E> Stage<'a, I> for DefaultMutationalStage<'a, I, R, E>
where
    I: Input,
    R: Rand,
    E: Evaluator<I>,
{
    fn perform(&mut self, entry: Rc<RefCell<Testcase<I>>>) -> Result<(), AflError> {
        self.perform_mutational(entry)
    }
}

impl<'a, I, R, E> DefaultMutationalStage<'a, I, R, E>
where
    I: Input,
    R: Rand,
    E: Evaluator<I>,
{
    pub fn new(rand: &Rc<RefCell<R>>, eval: &'a mut E) -> Self {
        DefaultMutationalStage {
            rand: Rc::clone(rand),
            eval: eval,
            mutators: vec![],
        }
    }
}
