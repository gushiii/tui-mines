use std::{
    fs,
    time::{Instant, SystemTime},
};

use serde::{Deserialize, Serialize};

// ==========================================
// MODEL: 游戏数据与核心逻辑
// ==========================================
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Score {
    pub date: String,
    pub seconds: u64,
    pub difficulty: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Leaderboard {
    pub scores: Vec<Score>,
}

impl Leaderboard {
    pub fn load() -> Self {
        fs::read_to_string("scores.json")
            .and_then(|content| Ok(serde_json::from_str(&content).unwrap_or_default()))
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = fs::write("scores.json", content);
        }
    }

    pub fn add_score(&mut self, seconds: u64, difficulty: String) {
        let date = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
        self.scores.push(Score {
            date,
            seconds,
            difficulty,
        });
        // 按时间排序，只保留前 10 名
        self.scores.sort_by_key(|s| s.seconds);
        self.scores.truncate(10);
        self.save();
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum CellState {
    Closed,
    Opened,
    Flagged,
}

pub struct MineCell {
    pub is_mine: bool,
    pub neighbor_mines: u8,
    pub state: CellState,
}

pub struct MinesweeperModel {
    pub grid: Vec<Vec<MineCell>>,
    pub cursor: (usize, usize),
    pub game_over: bool,
    pub won: bool,
    pub leaderboard: Leaderboard,
    pub width: usize,
    pub height: usize,
    pub mine_count: usize,
    pub first_click: bool,
    pub start_time: Option<Instant>,
    pub elapsed_time: u64,
}

impl MinesweeperModel {
    pub fn new_with_difficulty(width: usize, height: usize, difficulty: f32) -> Self {
        // difficulty: 0.1 (简单), 0.15 (普通), 0.2 (困难)
        let mine_count = (width * height) as f32 * difficulty;
        Self::new(width, height, mine_count as usize)
    }

    pub fn new(width: usize, height: usize, mine_count: usize) -> Self {
        let grid = (0..height)
            .map(|_| {
                (0..width)
                    .map(|_| MineCell {
                        is_mine: false,
                        neighbor_mines: 0,
                        state: CellState::Closed,
                    })
                    .collect()
            })
            .collect();

        Self {
            grid,
            cursor: (0, 0),
            game_over: false,
            won: false,
            leaderboard: Leaderboard::load(),
            width,
            height,
            mine_count,
            first_click: true,
            start_time: None,
            elapsed_time: 0,
        }
    }

    // 修改 reset：重置时间
    pub fn reset(&mut self) {
        let new_game = Self::new(self.width, self.height, self.mine_count);
        *self = new_game; // 直接替换整个 Model 更加简洁
    }

    // 核心保护逻辑：生成地雷，但避开 (safe_x, safe_y) 及其周围 8 个格子
    fn generate_mines(&mut self, safe_x: usize, safe_y: usize) {
        let mut pos: Vec<(usize, usize)> = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                // 避开点击位置及其邻居（3x3 区域），确保第一下点开的是空白或 0
                let is_safe_zone = (x as isize - safe_x as isize).abs() <= 1
                    && (y as isize - safe_y as isize).abs() <= 1;
                if !is_safe_zone {
                    pos.push((x, y));
                }
            }
        }

        // 洗牌埋雷
        let mut seed = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        for i in 0..pos.len() {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let j = (seed as usize) % pos.len();
            pos.swap(i, j);
        }

        for i in 0..self.mine_count.min(pos.len()) {
            let (px, py) = pos[i];
            self.grid[py][px].is_mine = true;
        }

        // 重新计算全图数字
        self.update_neighbor_counts();
    }

    fn update_neighbor_counts(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                if self.grid[y][x].is_mine {
                    continue;
                }
                let mut count = 0;
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let ny = y as isize + dy;
                        let nx = x as isize + dx;
                        if ny >= 0
                            && ny < self.height as isize
                            && nx >= 0
                            && nx < self.width as isize
                        {
                            if self.grid[ny as usize][nx as usize].is_mine {
                                count += 1;
                            }
                        }
                    }
                }
                self.grid[y][x].neighbor_mines = count;
            }
        }
    }

    fn open_all_mines(&mut self) {
        Self::play_sound("boom");
        for row in self.grid.iter_mut() {
            for cell in row.iter_mut() {
                if cell.is_mine {
                    cell.state = CellState::Opened;
                }
            }
        }
    }

    pub fn open_cell(&mut self) {
        let (x, y) = self.cursor;
        if self.grid[y][x].state != CellState::Closed {
            return;
        }

        if self.first_click {
            self.generate_mines(x, y);
            self.first_click = false;
            self.start_time = Some(std::time::Instant::now());
        }

        if self.grid[y][x].is_mine {
            // 标记游戏结束
            self.game_over = true;
            // 强制翻开当前这颗雷，让 View 能渲染它
            self.grid[y][x].state = CellState::Opened;
            // 翻开全图所有的地雷，让玩家死个明白
            self.open_all_mines();
        } else {
            Self::play_sound("click");
            self.recursive_open(x, y);
            self.check_win();
        }
    }

    pub fn chord_cell(&mut self) {
        let (x, y) = self.cursor;
        let cell = &self.grid[y][x];

        // 只有已翻开且有数字的格子才能触发 Chording
        if cell.state != CellState::Opened || cell.neighbor_mines == 0 || cell.is_mine {
            return;
        }

        Self::play_sound("click");
        // 计算周围已插旗的数量
        let mut flags = 0;
        for dy in -1..=1 {
            for dx in -1..=1 {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx >= 0 && ny >= 0 && nx < self.width as isize && ny < self.height as isize {
                    if self.grid[ny as usize][nx as usize].state == CellState::Flagged {
                        flags += 1;
                    }
                }
            }
        }

        // 如果旗帜数等于数字，翻开周围所有非插旗的 Closed 格子
        if flags == cell.neighbor_mines {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let nx_isize = x as isize + dx;
                    let ny_isize = y as isize + dy;

                    if nx_isize >= 0 && ny_isize >= 0 {
                        let nx = nx_isize as usize;
                        let ny = ny_isize as usize;

                        if nx < self.width && ny < self.height {
                            // 判断如果是雷，触发 game_over
                            if self.grid[ny][nx].state == CellState::Closed {
                                if self.grid[ny][nx].is_mine {
                                    self.game_over = true;
                                    // 踩雷后翻开所有雷
                                    self.open_all_mines();
                                } else {
                                    self.recursive_open(nx, ny);
                                }
                            }
                        }
                    }
                }
            }
            self.check_win();
        }
    }

    // 更新已用时间
    pub fn update_timer(&mut self) {
        if let Some(start) = self.start_time {
            if !self.game_over && !self.won {
                self.elapsed_time = start.elapsed().as_secs();
            }
        }
    }

    fn recursive_open(&mut self, x: usize, y: usize) {
        if x >= self.width || y >= self.height || self.grid[y][x].state != CellState::Closed {
            return;
        }
        self.grid[y][x].state = CellState::Opened;
        if self.grid[y][x].neighbor_mines == 0 && !self.grid[y][x].is_mine {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    if nx >= 0 && ny >= 0 {
                        self.recursive_open(nx as usize, ny as usize);
                    }
                }
            }
        }
    }

    pub fn toggle_flag(&mut self) {
        let current_flags = self.flags_count();
        let (x, y) = self.cursor;
        let cell = &mut self.grid[y][x];
        match cell.state {
            CellState::Closed => {
                if current_flags < self.mine_count {
                    cell.state = CellState::Flagged;
                    Self::play_sound("flag");
                }
            }
            CellState::Flagged => {
                cell.state = CellState::Closed;
                Self::play_sound("unflag");
            }
            _ => {}
        }
    }

    pub fn move_cursor(&mut self, dx: isize, dy: isize) {
        let nx = self.cursor.0 as isize + dx;
        let ny = self.cursor.1 as isize + dy;
        if nx >= 0 && nx < self.width as isize && ny >= 0 && ny < self.height as isize {
            self.cursor = (nx as usize, ny as usize);
        }
    }

    fn check_win(&mut self) {
        let all_safe_opened = self
            .grid
            .iter()
            .flatten()
            .all(|c| c.is_mine || c.state == CellState::Opened);
        if all_safe_opened && !self.won {
            self.won = true;
            Self::play_sound("win");
            // 获胜时记录成绩
            let diff_str = format!("{}x{}", self.width, self.height);
            self.leaderboard.add_score(self.elapsed_time, diff_str);
        }
    }

    pub fn flags_count(&self) -> usize {
        self.grid
            .iter()
            .flatten()
            .filter(|c| c.state == CellState::Flagged)
            .count()
    }

    // 播放音效辅助函数
    fn play_sound(effect: &str) {
        use std::process::Command;

        // 根据操作系统选择不同的命令
        #[cfg(target_os = "macos")]
        let (cmd, args) = match effect {
            "click" => ("afplay", "/System/Library/Sounds/Tink.aiff"),
            "boom" => ("afplay", "/System/Library/Sounds/Basso.aiff"),
            "win" => ("afplay", "/System/Library/Sounds/Glass.aiff"),
            "flag" => ("afplay", "/System/Library/Sounds/Pop.aiff"),
            "unflag" => ("afplay", "/System/Library/Sounds/Bottle.aiff"),
            _ => return,
        };

        #[cfg(target_os = "linux")]
        let (cmd, args) = match effect {
            "click" => (
                "paplay",
                "/usr/share/sounds/freedesktop/stereo/button-pressed.oga",
            ),
            "boom" => (
                "paplay",
                "/usr/share/sounds/freedesktop/stereo/dialog-error.oga",
            ),
            "win" => (
                "paplay",
                "/usr/share/sounds/freedesktop/stereo/complete.oga",
            ),
            "flag" => (
                "paplay",
                "/usr/share/sounds/freedesktop/stereo/message-new-instant.oga",
            ),
            "unflag" => (
                "paplay",
                "/usr/share/sounds/freedesktop/stereo/dialog-information.oga",
            ),
            _ => return,
        };

        #[cfg(target_os = "windows")]
        let (cmd, args) = match effect {
            // Windows 可以调用 powershell 的 Beep 或系统声音
            "click" => ("powershell", "[console]::beep(1000, 50)"),
            "boom" => ("powershell", "[console]::beep(200, 300)"),
            "win" => ("powershell", "[console]::beep(1500, 500)"),
            "flag" => ("powershell", "[console]::beep(800, 50)"),
            "unflag" => ("powershell", "[console]::beep(600, 50)"),
            _ => return,
        };

        // 异步启动进程，不等待它结束
        let _ = Command::new(cmd).arg(args).spawn();
    }
}
