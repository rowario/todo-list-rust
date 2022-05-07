use tui::{
    backend::Backend,
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Spans,
    widgets::{BarChart, Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};
use unicode_width::UnicodeWidthStr;
use crate::{App, Screen};

pub fn todos_screen<B: Backend>(app: &App, f: &mut Frame<B>, todos: bool) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(30),
                Constraint::Percentage(70)
            ].as_ref()
        )
        .split(f.size());
    todos_block(app, f, todos, chunks[0]);
    notes_block(app, f, !todos && !matches!(app.screen,Screen::NewTodo), chunks[1]);
}

fn todos_block<B: Backend>(app: &App, f: &mut Frame<B>, active: bool, area: Rect) {
    let block = Block::default()
        .title(format!("TODOs | {}", app.day.date))
        .borders(Borders::ALL).style(Style::default().fg(if active { Color::Yellow } else { Color::White }));
    let list = List::new(get_todos_list(app, active)).block(block);
    f.render_widget(list, area);
}

fn notes_block<B: Backend>(app: &App, f: &mut Frame<B>, active: bool, area: Rect) {
    let text = String::from(&app.day.notes);
    let text: Vec<Spans> = text.split('\n').map(|s| {
        Spans::from(s.trim_start())
    }).collect();
    let block = Block::default()
        .title(format!("Notes{}", if matches!(app.screen, Screen::EditNotes) { "*" } else { "" }))
        .borders(Borders::ALL)
        .style(Style::default().fg(if active { Color::Yellow } else { Color::White }));
    let text_block = Paragraph::new(text.clone())
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::White))
        .block(block);
    f.render_widget(text_block, area);
    if matches!(app.screen,Screen::EditNotes) {
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

pub fn stats_screen<B: Backend>(_app: &App, f: &mut Frame<B>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(30),
                Constraint::Percentage(70)
            ].as_ref()
        )
        .split(f.size());
    let block = Block::default()
        .title("Days")
        .borders(Borders::ALL);
    let data: [(&str, u64); 5] = [("*27.04.2022*", 100), ("-28.04.2022-", 90), ("29.04.2022", 80), ("30.04.2022", 20), ("01.05.2022", 40)];
    let chart = BarChart::default()
        .bar_width(12)
        .bar_style(Style::default().fg(Color::Yellow))
        .value_style(Style::default().fg(Color::White).bg(Color::Yellow))
        .label_style(Style::default().fg(Color::White))
        .data(&data)
        .block(block);
    f.render_widget(chart, chunks[0]);
    let block = Block::default()
        .title("Notes")
        .borders(Borders::ALL);
    f.render_widget(block, chunks[1]);
}

pub fn new_screen<B: Backend>(app: &App, f: &mut Frame<B>) {
    let block = Paragraph::new(app.input.as_ref()).style(Style::default().fg(Color::White))
        .block(Block::default().title("New TODO").borders(Borders::ALL).style(Style::default().fg(Color::Yellow)));
    let area = centered_input(60, f.size());
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    f.set_cursor(area.x + app.input.width() as u16 + 1, area.y + 1);
}

pub fn get_todos_list(app: &App, active: bool) -> Vec<ListItem> {
    app.day.todos.iter().enumerate().map(|(index, todo)| {
        ListItem::new(todo.get_text())
            .style(Style::default().fg(if index == app.index && active { Color::Yellow } else { Color::White }))
    }).collect()
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
