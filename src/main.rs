use inquire::Select;
use rand::Rng;
use std::cmp::{max, min};
use std::fmt::Display;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Player {
    X,
    O,
}

impl Player {
    fn opponent(self) -> Self {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
}

impl Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Player::X => 'X',
                Player::O => 'O',
            }
        )
    }
}

// Hack to have inquire display a nice name for the board index representing the place on the board
#[derive(Copy, Clone, Debug)]
struct Selection {
    pub place: usize,
}

impl Selection {
    const PLACES: [&'static str; 9] = [
        "Top Left",
        "Top Middle",
        "Top Right",
        "Middle Left",
        "Middle",
        "Middle Right",
        "Bottom Left",
        "Bottom Middle",
        "Bottom Right",
    ];

    fn new(place: usize) -> Self {
        Self { place }
    }
}

impl Display for Selection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::PLACES[self.place])
    }
}

fn get_char(square: Option<Player>) -> char {
    match square {
        Some(Player::X) => 'X',
        Some(Player::O) => 'O',
        None => '.',
    }
}

fn get_line_winner(a: Option<Player>, b: Option<Player>, c: Option<Player>) -> Option<Player> {
    if a != None && a == b && b == c {
        a
    } else {
        None
    }
}

#[derive(Clone)]
struct GameState {
    board: [Option<Player>; 9],
    next_player: Player,
}

fn minimax(state: &GameState, maximizing_player: Player) -> i32 {
    if let Some(winner) = state.winner() {
        if winner == maximizing_player {
            return 10;
        } else {
            return -10;
        }
    }

    let possible_moves = state.open_squares();
    if possible_moves.len() == 0 {
        return 0;
    }

    if state.next_player == maximizing_player {
        let mut best_so_far = -100;
        for m in possible_moves {
            best_so_far = max(
                best_so_far,
                minimax(&state.with_move(m.place), maximizing_player),
            );
        }
        return best_so_far;
    } else {
        let mut best_so_far = 100;
        for m in possible_moves {
            best_so_far = min(
                best_so_far,
                minimax(&state.with_move(m.place), maximizing_player),
            );
        }
        return best_so_far;
    }
}

impl GameState {
    fn new() -> Self {
        Self {
            board: [None; 9],
            next_player: Player::X,
        }
    }

    fn apply_move(&mut self, place: usize) {
        self.board[place] = Some(self.next_player);
        self.next_player = self.next_player.opponent();
    }

    fn with_move(&self, place: usize) -> Self {
        let mut new_state = self.clone();
        new_state.apply_move(place);
        new_state
    }

    fn smart_choice(&self, player: Player) -> Selection {
        let possible_moves = self.open_squares();

        let mut best_so_far = -100;

        let mut winning_moves = Vec::new();

        for m in possible_moves {
            let move_score = minimax(&self.with_move(m.place), player);

            if move_score > best_so_far {
                best_so_far = move_score;
                winning_moves = Vec::new();
                winning_moves.push(m);
            } else if move_score == best_so_far {
                winning_moves.push(m);
            }
        }

        // Randomly choose one of the best moves to avoid repetitive games
        let mut rng = rand::thread_rng();
        winning_moves[rng.gen_range(0..winning_moves.len())]
    }

    fn open_squares(&self) -> Vec<Selection> {
        self.board
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                if s.is_none() {
                    Some(Selection::new(i))
                } else {
                    None
                }
            })
            .collect()
    }

    fn winner(&self) -> Option<Player> {
        for a in 0..3 {
            // Check rows
            if let Some(winner) = get_line_winner(
                self.board[a * 3],
                self.board[a * 3 + 1],
                self.board[a * 3 + 2],
            ) {
                return Some(winner);
            }

            // Check columns
            if let Some(winner) =
                get_line_winner(self.board[a], self.board[a + 3], self.board[a + 6])
            {
                return Some(winner);
            }
        }

        // Check diagonals
        if let Some(winner) = get_line_winner(self.board[0], self.board[4], self.board[8]) {
            return Some(winner);
        }

        if let Some(winner) = get_line_winner(self.board[2], self.board[4], self.board[6]) {
            return Some(winner);
        }

        None
    }
}

impl ToString for GameState {
    fn to_string(&self) -> String {
        let mut s = String::new();
        for x in (0..9).step_by(3) {
            s.push_str(&format!(
                " {} | {} | {} \n",
                get_char(self.board[x]),
                get_char(self.board[x + 1]),
                get_char(self.board[x + 2]),
            ));
            if x != 6 {
                s.push_str(&format!("---|---|---\n"));
            }
        }
        s
    }
}

fn main() -> anyhow::Result<()> {
    let mut game = GameState::new();
    let mut winner = None;

    let user_player = Select::new("Will you play X or O?", vec![Player::X, Player::O]).prompt()?;

    loop {
        let possible_moves = game.open_squares();
        if possible_moves.len() == 0 {
            break;
        }

        let selection = if game.next_player == user_player {
            println!("{}", game.to_string());
            let page_size = possible_moves.len();
            let selection = Select::new("Where will you move?", possible_moves)
                .with_page_size(page_size)
                .prompt()?;
            selection.place
        } else {
            let computer_selection = game.smart_choice(user_player.opponent());
            println!("Computer moved to {}", computer_selection);
            computer_selection.place
        };

        game.apply_move(selection);

        winner = game.winner();
        if winner.is_some() {
            break;
        }
    }

    println!("{}", game.to_string());

    if winner == Some(user_player) {
        println!("Congratulations, you won!");
    } else if winner == Some(user_player.opponent()) {
        println!("You lost, better luck next time.");
    } else {
        println!("The game ended in a tie.");
    }

    Ok(())
}
