use crate::evm::Stack;
use nes_yew::*;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct StackProps {
    pub stack: Stack,
}

#[function_component(StackViewer)]
pub fn stack_viewer(StackProps { stack }: &StackProps) -> Html {
    html! {
        <List>
            {
                stack.inner.iter().map(|value| {
                    html! {
                        <li>{ value }</li>
                    }
                }).collect::<Html>()
            }
        </List>
    }
}
