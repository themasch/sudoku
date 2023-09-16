use std::num::NonZeroU8;

use crate::solver::{GenerateBasicMarkingsStep, NakedSingleStep};
use log::info;
use yew::prelude::*;

mod solver;
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

    let solver = use_mut_ref(|| {
        let mut solver = solver::Solver::default();
        solver.add_step(GenerateBasicMarkingsStep);
        solver.add_step(NakedSingleStep);
        solver
    });

    let on_solver_step = {
        let game = game.clone();
        let solver = solver.clone();
        Callback::from(move |_| {
            let cgame = (*solver).borrow_mut().next_step(*game);
            game.set(cgame);
        })
    };

    let on_number_input = {
        let game = game.clone();
        Callback::from(
            move |(row, col, value, is_marking): (usize, usize, u8, bool)| {
                let mut cgame = *game.clone();
                if !is_marking {
                    cgame.set(row, col, value);
                } else {
                    info!("toggling marking: {:?}", (row, col, value));
                    cgame.toggle_note(row, col, value);
                }

                game.set(cgame);
            },
        )
    };

    html! {
        <div>
            <button onclick={on_solver_step}>{ "run solver step" }</button>
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
    number_input: Callback<(usize, usize, u8, bool), ()>,
}

#[function_component]
fn Field(props: &FieldProps) -> Html {
    let selected = use_state_eq(|| None);

    let create_keyboard_input =
        |row: usize,
         col: usize,
         number_input: Callback<_>,
         selected: UseStateHandle<Option<usize>>| {
            return move |keyboard_event: KeyboardEvent| {
                keyboard_event.prevent_default();
                keyboard_event.stop_propagation();
                keyboard_event.cancel_bubble();

                let input = keyboard_event.key_code();
                match input {
                    46 /* del */ => {
                        number_input.emit((row, col, 0, false));
                    }
                    48..=57 => {
                        let input_val = (input - 48) as u8;
                        number_input.emit((row, col, input_val, keyboard_event.ctrl_key()));
                    }
                    _ => info!("no mapping for key code {}", keyboard_event.key_code()),
                }
            };
        };

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
                        let selected = selected.clone();
                        Callback::from(create_keyboard_input(row, col, number_input, selected))
                    };

                    let fixed = props.game.index_is_given(game_index);
                    let value = match value {
                        0 => None,
                        x => Some(x.try_into().unwrap())
                    };

                    let markings = if value.is_none() {
                        match props.game.get_notes(row, col) {
                            0 => None,
                            x => {
                                Some([
                                    /* 1 */ x & 0x0001 > 0,
                                    /* 2 */ x & 0x0002 > 0,
                                    /* 3 */ x & 0x0004 > 0,
                                    /* 4 */ x & 0x0008 > 0,
                                    /* 5 */ x & 0x0010 > 0,
                                    /* 6 */ x & 0x0020 > 0,
                                    /* 7 */ x & 0x0040 > 0,
                                    /* 8 */ x & 0x0080 > 0,
                                    /* 9 */ x & 0x0100 > 0,
                                ])
                            }
                        }
                    } else {
                        None
                    };

                    html! {
                        <Cell idx={idx as u8} onkeyup={onkeyup} onfocus={on_cell_select} selected={ Some(idx) == *selected } fixed={fixed} value={value} markings={markings} />
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
    #[prop_or_default]
    markings: Option<[bool; 9]>,
}

#[function_component]
fn Cell(props: &CellProps) -> Html {
    let idx = format!("{}", props.idx);

    let content = if let Some(value) = props.value {
        html! { value }
    } else if let Some(markings) = props.markings {
        html! {
            <div class="markings">
                <div> { if markings[0] { "1" } else { "" } } </div>
                <div> { if markings[1] { "2" } else { "" } } </div>
                <div> { if markings[2] { "3" } else { "" } } </div>
                <div> { if markings[3] { "4" } else { "" } } </div>
                <div> { if markings[4] { "5" } else { "" } } </div>
                <div> { if markings[5] { "6" } else { "" } } </div>
                <div> { if markings[6] { "7" } else { "" } } </div>
                <div> { if markings[7] { "8" } else { "" } } </div>
                <div> { if markings[8] { "9" } else { "" } } </div>
            </div>
        }
    } else {
        html! {}
    };

    html! {
        <div tabindex={idx}
            onfocus={props.onfocus.clone()}
            onkeyup={props.onkeyup.clone()}
            class={classes!("cell", props.selected.then_some("selected"), props.fixed.then_some("fixed"))}
            >
            { content }
        </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<Game>::new().render();
}
