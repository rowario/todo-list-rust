use crate::{App, Screen};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Spans,
    widgets::{BarChart, Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use unicode_width::UnicodeWidthStr;

pub fn todos_screen<B: Backend>(app: &App, f: &mut Frame<B>, todos: bool) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(f.size());
    todos_block(app, f, todos, chunks[0]);
    let active_notes = !todos
        && !matches!(
            app.screen,
            Screen::NewTodo | Screen::DailyTodos | Screen::NewDailyTodo
        );
    notes_block(app, f, &app.day.notes, active_notes, chunks[1]);
}

fn todos_block<B: Backend>(app: &App, f: &mut Frame<B>, active: bool, area: Rect) {
    let block = Block::default()
        .title(format!("TODOs | {}", app.day.date))
        .borders(Borders::ALL)
        .style(Style::default().fg(if active { Color::Yellow } else { Color::White }));
    let list = List::new(get_todos_list(app, active)).block(block);
    f.render_widget(list, area);
}

fn notes_block<B: Backend>(app: &App, f: &mut Frame<B>, text: &str, active: bool, area: Rect) {
    let text = String::from(text);
    let text: Vec<Spans> = text
        .split('\n')
        .map(|s| Spans::from(s.trim_start()))
        .collect();
    let block = Block::default()
        .title(format!(
            "Notes{}",
            if matches!(app.screen, Screen::EditNotes) {
                "*"
            } else {
                ""
            }
        ))
        .borders(Borders::ALL)
        .style(Style::default().fg(if active { Color::Yellow } else { Color::White }));
    let text_block = Paragraph::new(text.clone())
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::White))
        .block(block);
    f.render_widget(text_block, area);
    if matches!(app.screen, Screen::EditNotes) {
        let x = if !text.is_empty() {
            area.x + 1 + text.last().unwrap().width() as u16
        } else {
            area.x + 1
        };
        let y = if !text.is_empty() {
            area.y + text.len() as u16
        } else {
            area.y
        };
        f.set_cursor(x, y);
    }
}

pub fn daily_todos_screen<B: Backend>(app: &App, f: &mut Frame<B>, active: bool) {
    let block = List::new(get_daily_todos_list(app, active))
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .title("Daily TODOs")
                .borders(Borders::ALL)
                .style(Style::default().fg(if active { Color::Yellow } else { Color::White })),
        );
    let area = centered_rect(30, 50, f.size());
    f.render_widget(Clear, area);
    f.render_widget(block, area);
}

pub fn stats_screen<B: Backend>(app: &App, f: &mut Frame<B>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(f.size());
    let chunks2 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(chunks[1]);
    let block = Block::default()
        .title("Days")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));
    let data: Vec<(String, u64)> = app
        .stats_list
        .list
        .iter()
        .enumerate()
        .map(|(index, d)| {
            (
                (if index == app.stats_list.index {
                    format!("*{}*", &d.date)
                } else {
                    format!("-{}-", &d.date)
                }),
                d.done as u64,
            )
        })
        .collect();
    let data: Vec<(&str, u64)> = data.iter().map(|d| (d.0.as_str(), d.1 as u64)).collect();
    let chart = BarChart::default()
        .bar_width(12)
        .bar_style(Style::default().fg(Color::Yellow))
        .value_style(Style::default().fg(Color::White).bg(Color::Yellow))
        .label_style(Style::default().fg(Color::White))
        .data(&data)
        .block(block);
    f.render_widget(chart, chunks[0]);
    let current_day = app.stats_list.get_current(&app.db).unwrap();
    let todos_list: Vec<ListItem> = current_day
        .todos
        .iter()
        .map(|todo| ListItem::new(todo.get_text()))
        .collect();
    let todos_block = List::new(todos_list).block(
        Block::default()
            .title(format!("TODOs | {}", current_day.date))
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White)),
    );
    f.render_widget(todos_block, chunks2[0]);
    notes_block(app, f, &current_day.notes, false, chunks2[1]);
}

pub fn new_daily_todo_screen<B: Backend>(app: &App, f: &mut Frame<B>) {
    let block = Paragraph::new(app.daily_todos.input.as_ref())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .title("New Daily TODO")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Yellow)),
        );
    let area = centered_input(60, f.size());
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    f.set_cursor(
        area.x + app.daily_todos.input.width() as u16 + 1,
        area.y + 1,
    );
}

pub fn new_todo_screen<B: Backend>(app: &App, f: &mut Frame<B>) {
    let block = Paragraph::new(app.input.as_ref())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .title("New TODO")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Yellow)),
        );
    let area = centered_input(60, f.size());
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    f.set_cursor(area.x + app.input.width() as u16 + 1, area.y + 1);
}

pub fn get_todos_list(app: &App, active: bool) -> Vec<ListItem> {
    app.day
        .todos
        .iter()
        .enumerate()
        .map(|(index, todo)| {
            ListItem::new(todo.get_text()).style(Style::default().fg(
                if index == app.index && active {
                    Color::Yellow
                } else {
                    Color::White
                },
            ))
        })
        .collect()
}

pub fn get_daily_todos_list(app: &App, active: bool) -> Vec<ListItem> {
    app.daily_todos
        .list
        .iter()
        .enumerate()
        .map(|(index, todo)| {
            ListItem::new(todo.get_text()).style(Style::default().fg(
                if index == app.daily_todos.index && active {
                    Color::Yellow
                } else {
                    Color::White
                },
            ))
        })
        .collect()
}

pub fn centered_input(percent_x: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(r.height / 2 - 1),
                Constraint::Min(3),
                Constraint::Length(r.height / 2 - 1),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
