use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use std::io::{self, stdout};

use crate::{
    model::{CellState, MinesweeperModel},
    view::MinesweeperView,
};

mod model;
mod view;

// ==========================================
// CONTROLLER: 驱动循环与事件处理
// ==========================================
const EASY: (usize, usize, f32) = (10, 10, 0.15); // 10x10: 15 颗雷（15%）
const MEDIUM: (usize, usize, f32) = (16, 16, 0.15625); // 16x16: 40 颗雷（15.6%）
const HARD: (usize, usize, f32) = (30, 16, 0.20625); // 30x16: 99 颗雷（20.6%）

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut model = MinesweeperModel::new_with_difficulty(EASY.0, EASY.1, EASY.2);

    loop {
        model.update_timer();
        terminal.draw(|f| MinesweeperView::draw(f, &model))?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                // 如果游戏已结束
                if model.game_over || model.won {
                    match key.code {
                        KeyCode::Char('q') => break, // 按 Q 依然退出
                        _ => model.reset(),          // 按其他任意键重置游戏
                    }
                }
                // 如果游戏正在进行中
                else {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('1') => {
                            model = MinesweeperModel::new_with_difficulty(EASY.0, EASY.1, EASY.2)
                        }
                        KeyCode::Char('2') => {
                            model =
                                MinesweeperModel::new_with_difficulty(MEDIUM.0, MEDIUM.1, MEDIUM.2)
                        }
                        KeyCode::Char('3') => {
                            model = MinesweeperModel::new_with_difficulty(HARD.0, HARD.1, HARD.2)
                        }
                        KeyCode::Left => model.move_cursor(-1, 0),
                        KeyCode::Right => model.move_cursor(1, 0),
                        KeyCode::Up => model.move_cursor(0, -1),
                        KeyCode::Down => model.move_cursor(0, 1),
                        KeyCode::Char(' ') => {
                            let (x, y) = model.cursor;
                            if model.grid[y][x].state == CellState::Opened {
                                model.chord_cell(); // 如果已翻开，尝试 Chording
                            } else {
                                model.open_cell(); // 如果未翻开，正常翻开
                            }
                        }
                        KeyCode::Char('f') => model.toggle_flag(),
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
