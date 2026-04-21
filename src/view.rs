use ratatui::{prelude::*, widgets::*};

use crate::model::{CellState, MinesweeperModel};

// ==========================================
// VIEW: 负责渲染界面
// ==========================================
pub struct MinesweeperView;

fn get_centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.width.saturating_sub(width) / 2;
    let y = area.height.saturating_sub(height) / 2;
    Rect::new(
        area.x + x,
        area.y + y,
        width.min(area.width),
        height.min(area.height),
    )
}

impl MinesweeperView {
    // 辅助函数：根据文本内容计算居中且自适应大小的矩形
    fn get_adaptive_rect(text: &str, screen: Rect) -> Rect {
        let lines: Vec<&str> = text.lines().collect();
        // 计算文本中最长一行的宽度
        let content_width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16;
        let content_height = lines.len() as u16;

        // 加上边框和内边距 (左右+4, 上下+2)
        let width = (content_width + 4).min(screen.width);
        let height = (content_height + 2).min(screen.height);

        let x = (screen.width - width) / 2;
        let y = (screen.height - height) / 2;

        Rect::new(x, y, width, height)
    }

    pub fn draw(f: &mut Frame, model: &MinesweeperModel) {
        let area = f.area();

        let chunks = Layout::vertical([
            Constraint::Length(1), // 顶部信息
            Constraint::Min(0),    // 棋盘
            Constraint::Length(1), // 底部指南
        ])
        .split(area);

        // 构建 Table 的行数据
        let rows: Vec<Row> = model
            .grid
            .iter()
            .enumerate()
            .map(|(y, row_data)| {
                let cells: Vec<ratatui::widgets::Cell> = row_data
                    .iter()
                    .enumerate()
                    .map(|(x, cell)| {
                        // 判断是否是光标位置
                        let is_cursor = (x, y) == model.cursor;
                        // 判断是否处于光标周围的 8 个格（辅助框范围）
                        let is_neighbor = (x as isize - model.cursor.0 as isize).abs() <= 1
                            && (y as isize - model.cursor.1 as isize).abs() <= 1
                            && !is_cursor; // 排除中心光标

                        // 获取内容和基本样式
                        let (content, color) = match cell.state {
                            CellState::Closed => {
                                (" \u{f0764} ".to_string(), Color::Rgb(100, 100, 100))
                            }
                            CellState::Flagged => {
                                (" \u{f024}".to_string(), Color::Rgb(255, 80, 80))
                            }
                            CellState::Opened => {
                                if cell.is_mine {
                                    (" \u{f0691}".to_string(), Color::Rgb(255, 0, 0))
                                } else if cell.neighbor_mines == 0 {
                                    (" . ".to_string(), Color::Rgb(80, 80, 80))
                                } else {
                                    let c = match cell.neighbor_mines {
                                        1 => Color::Rgb(100, 150, 255),
                                        2 => Color::Rgb(100, 255, 100),
                                        3 => Color::Rgb(255, 100, 100),
                                        4 => Color::Rgb(150, 100, 255),
                                        _ => Color::Magenta,
                                    };
                                    (format!(" {} ", cell.neighbor_mines), c)
                                }
                            }
                        };

                        let mut style = Style::default().fg(color);
                        if is_cursor {
                            // 光标中心：保持之前的醒目黄色
                            style = style.bg(Color::Yellow).fg(Color::Black).bold();
                        } else if is_neighbor {
                            // 辅助框范围：设置一个淡淡的背景色或改变边框感
                            style = style.bg(Color::Rgb(60, 60, 60));
                        }

                        // 返回 ratatui 的 Cell
                        ratatui::widgets::Cell::from(content).style(style)
                    })
                    .collect();

                Row::new(cells).height(1)
            })
            .collect();

        // let remaining = model.remaining_mines();
        let time_info = format!(
            " \u{f13ab}: {:02}:{:02} ",
            model.elapsed_time / 60,
            model.elapsed_time % 60
        );
        let title = format!(" | \u{f024}: {}/{}", model.flags_count(), model.mine_count,);
        let info_line = Line::from(vec![
            Span::styled(" Minesweeper ", Style::default().fg(Color::Cyan)),
            Span::raw(title),
            Span::styled(
                format!(" | {} ", time_info),
                Style::default().fg(Color::Yellow),
            ),
        ]);
        f.render_widget(
            Paragraph::new(info_line).alignment(Alignment::Center),
            chunks[0],
        );

        let board_width = (model.width * 3 + 2) as u16;
        let board_height = (model.height + 2) as u16;
        let board_area = get_centered_rect(board_width, board_height, chunks[1]);

        let widths = vec![Constraint::Length(3); model.grid[0].len()];
        let table = Table::new(rows, widths)
            .block(Block::bordered()) // 标题简短
            .column_spacing(0);
        f.render_widget(table, board_area);

        f.render_widget(
            Paragraph::new("1-3: Difficulty | Space: Open/Chord | F: Flag | Q: Quit")
                .alignment(Alignment::Center),
            chunks[2],
        );

        if model.game_over || model.won {
            let screen_area = f.area();

            let text = if model.won {
                let mut top_scores = "\u{f091} TOP SCORES \u{f091}\n".to_string();
                for (i, s) in model.leaderboard.scores.iter().take(3).enumerate() {
                    top_scores.push_str(&format!("{}. {}s - {}\n", i + 1, s.seconds, s.date));
                }
                format!(
                    "{}\n\u{f05d5} YOU WIN! {:02}:{:02}\nPress any key to restart",
                    top_scores,
                    model.elapsed_time / 60,
                    model.elapsed_time % 60
                )
            } else {
                "\u{f0691} GAME OVER \u{f0691}\nPress any key to restart".to_string()
            };

            // 2. 核心修复：显式借用为 &str
            let popup_area = Self::get_adaptive_rect(&text, screen_area);

            let popup = Paragraph::new(text).alignment(Alignment::Center).block(
                Block::bordered()
                    .title(" Result ")
                    .bg(Color::Rgb(50, 50, 50)),
            );

            f.render_widget(Clear, popup_area);
            f.render_widget(popup, popup_area);
        }
    }
}
