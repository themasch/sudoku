use log::info;
use std::num::NonZeroU8;
use yew::prelude::*;
mod sudoku;

#[rustfmt::skip]
static TEST_FIELD: [u8; 81] = [
    1, 0, 0,  0, 6, 0,  0, 0, 0,
    9, 8, 0,  0, 0, 0,  6, 0, 5,
    0, 0, 0,  0, 0, 5,  0, 0, 1,

    0, 0, 0,  0, 0, 0,  3, 0, 4,
    0, 6, 0,  1, 3, 0,  9, 0, 0,
    0, 4, 0,  7, 2, 0,  0, 0, 0,

    0, 9, 3,  0, 7, 6,  1, 0, 0,
    0, 0, 6,  4, 8, 0,  0, 0, 7,
    5, 0, 0,  9, 0, 2,  4, 6, 0,
];

#[derive(PartialEq, Properties)]
struct FieldProps {
    given_numbers: [u8; 81],
}

#[function_component]
fn Field(props: &FieldProps) -> Html {
    let selected = use_state_eq(|| None);
    let input_numbers = use_state_eq(|| props.given_numbers);

    let is_fixed_field = {
        let given_numbers = props.given_numbers;
        move |index: usize| -> bool { given_numbers[index] != 0 }
    };

    let get_value_at = {
        let input_numbers = input_numbers.clone();
        move |index: usize| -> Option<NonZeroU8> {
            match props.given_numbers[index] {
                0 => match input_numbers[index] {
                    0 => None,
                    x => Some(x.try_into().unwrap()),
                },
                x => Some(x.try_into().unwrap()),
            }
        }
    };

    html! {
        <div class="field">
        {

            (1..=81)
                .map(| idx | {
                    let on_cell_select = {
                        let selected = selected.clone();
                        Callback::from(move |_| {
                            if *selected == Some(idx) {
                                selected.set(None);
                            } else {
                                selected.set(Some(idx));
                            }
                        })
                    };

                    let game_index = (idx - 1) as usize;
                    let onkeyup = {
                        let input_numbers = input_numbers.clone();
                        Callback::from(move | keyboard_event: KeyboardEvent | {
                            let input = keyboard_event.key_code();
                            if is_fixed_field(game_index) {
                                return;
                            }

                            // keys 0..9
                            if (48..=57).contains(&input) {
                                let input_val = (input - 48) as u8;
                                info!("inserting {:?}", input_val);
                                let mut new_numbers = *input_numbers;
                                new_numbers[game_index] = input_val;
                                input_numbers.set(new_numbers);
                            } else {
                                match input {
                                    46 /* del */ => {
                                        let mut new_numbers = *input_numbers;
                                        new_numbers[game_index] = 0;
                                        input_numbers.set(new_numbers);
                                    },
                                    _ => info!("no mapping for key code {}", keyboard_event.key_code())
                                };

                            }
                        })
                    };

                    let value = get_value_at(game_index);

                    html! {
                        <Cell idx={idx as u8} onkeyup={onkeyup} onfocus={on_cell_select} selected={ Some(idx) == *selected } fixed={is_fixed_field(game_index)} value={value} />
                    }
                })
                .collect::<Html>()

        }
        </div>
    }
}

#[derive(Debug, Properties, PartialEq, Default)]
pub struct CellProps {
    #[prop_or_default]
    value: Option<NonZeroU8>,
    onfocus: Callback<FocusEvent, ()>,
    onkeyup: Callback<KeyboardEvent, ()>,
    idx: u8,
    #[prop_or_default]
    selected: bool,
    #[prop_or_default]
    fixed: bool,
}

#[function_component]
fn Cell(props: &CellProps) -> Html {
    let idx = format!("{}", props.idx);

    html! {
        <div tabindex={idx}
            onfocus={props.onfocus.clone()}
            onkeyup={props.onkeyup.clone()}
            class={classes!("cell", props.selected.then_some("selected"), props.fixed.then_some("fixed"))}
            >
            { props.value }
        </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<Field>::with_props(FieldProps {
        given_numbers: TEST_FIELD,
    })
    .render();
}
