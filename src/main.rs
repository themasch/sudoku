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

#[function_component]
fn Game() -> Html {
    let game = use_state_eq(|| sudoku::Game::create(TEST_FIELD));

    let on_number_input = {
        let game = game.clone();
        Callback::from(move |(row, col, value)| {
            let mut cgame = *game.clone();
            cgame.set(row, col, value);
            game.set(cgame);
        })
    };

    html! {
        <div>
            <Field game={*game} number_input={on_number_input} />
            <div>
                {
                    if game.is_valid() {
                        html!{ "game is valid" }
                    } else {
                        html!{ "game is INvalid" }
                    }
                }
            </div>
        </div>
    }
}

#[derive(PartialEq, Properties)]
struct FieldProps {
    game: sudoku::Game,
    number_input: Callback<(usize, usize, u8), ()>,
}

#[function_component]
fn Field(props: &FieldProps) -> Html {
    let selected = use_state_eq(|| None);

    html! {
        <div class="field">
        {

            props.game.cells().enumerate()
                .map(| (game_index, value) | {

                    let idx = game_index + 1;
                    let (row, col) = sudoku::Game::cell_index_to_coords(game_index);

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

                    let onkeyup = {
                        let number_input = props.number_input.clone();
                        Callback::from(move | keyboard_event: KeyboardEvent | {
                            let input = keyboard_event.key_code();
                            if (48..=57).contains(&input) {
                                let input_val = (input - 48) as u8;
                                info!("inserting {:?}", input_val);
                                number_input.emit((row, col, input_val));
                            } else {
                                match input {
                                    46 /* del */ => {
                                        number_input.emit((row, col, 0));
                                    },
                                    _ => info!("no mapping for key code {}", keyboard_event.key_code())
                                };

                            }
                        })
                    };

                    let fixed = props.game.index_is_given(game_index);
                    let value = match value {
                        0 => None,
                        x => Some(x.try_into().unwrap())
                    };

                    html! {
                        <Cell idx={idx as u8} onkeyup={onkeyup} onfocus={on_cell_select} selected={ Some(idx) == *selected } fixed={fixed} value={value} />
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
    yew::Renderer::<Game>::new().render();
}
