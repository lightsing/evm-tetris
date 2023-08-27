mod components;
mod evm;

use crate::components::*;
use crate::evm::*;
use rand::prelude::*;
use yew::prelude::*;

#[function_component]
fn App() -> Html {
    let rng = use_mut_ref(|| SmallRng::from_entropy());
    let program_counter = use_state(|| 0usize);
    let access_list = use_state(|| AccessList::default());
    let bytecode = use_state(|| Bytecode::default());
    let gas = use_state(|| Gas::new(10000));
    let memory = use_state(|| Memory::default());
    let stack = use_state(|| Stack::default());
    let storage = use_state(|| Storage::default());
    let next_instruction = use_state(|| Instruction::random(SmallRng::from_entropy()));
    let instruction_slots: UseStateHandle<[Option<Instruction>; 16]> = use_state(|| [None; 16]);

    html! {
        <div>
            <p>{(*next_instruction).clone()}</p>
            <StackViewer stack={(*stack).clone()} />
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
