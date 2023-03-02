use crate::core::coord::*;
use crate::core::ai::*;
use crate::core::board::*;
use crate::core::command::*;
use crate::core::entities::*;
use crate::core::state::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiAction {
    ActionAtCoord(HexCoord),
    Undo,
    AnimationFinished,
    AnimationInProgress,
    Idle,
    Busy,
    Restart,
}

pub trait View {
    fn request_update(&mut self);
    fn tick(&mut self, state: &State) -> UiAction;
}

pub struct Game {
    state: State,
    view: Box<dyn View>,
    human_player: Player,
    current_player: Player,
    ai: SimpleAI,
}

impl Game {
    pub fn new(human_player: Player, view: Box<dyn View>, board: Board, ai_depth: u32) -> Self {
        let mut game = Game {
            state: State::new(board),
            view,
            human_player,
            current_player: human_player,
            ai: SimpleAI::new(human_player.other(), ai_depth),
        };
        game.view.request_update();
        game
    }

    fn execute_for_coord(&mut self, coord: &HexCoord) -> bool {
        let some_move = self
            .state
            .legal_moves()
            .into_iter()
            .find(|m| m.coord() == *coord);

        if let Some(some_move) = some_move {
            if some_move.is_legal(&self.state) {
                some_move.execute(&mut self.state);
                return true;
            }
        }
        false
    }

    pub fn tick(&mut self) {
        let ui_action = self.view.tick(&mut self.state);

        if ui_action == UiAction::Busy {
            return;
        }

        if self.state.won_by().is_none() && self.current_player == self.human_player.other() {
            self.ai.turn(&mut self.state);
            self.view.request_update();
        }

        // ensure that the last move for the current player is rendered
        self.current_player = self.state.current_player;

        let successful_action = match ui_action {
            UiAction::ActionAtCoord(coord) => self.execute_for_coord(&coord),
            UiAction::Undo => {
                let p = self.state.current_player;
                loop {
                    let ret = self.state.undo();
                    if !ret || self.state.current_player == p {
                        break;
                    }
                }
                true
            }
            UiAction::Restart => {
                self.state.restart();
                true
            }
            _ => false,
        };

        if successful_action {
            self.view.request_update();
        } 
    }
}
