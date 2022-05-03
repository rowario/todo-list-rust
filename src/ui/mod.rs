use tui::backend::Backend;
use tui::Frame;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::widgets::{BarChart, Block, Borders, Clear, List, ListItem, Paragraph};
use unicode_width::UnicodeWidthStr;
use crate::App;

pub fn todos_screen<B: Backend>(app: &App, f: &mut Frame<B>, active: bool) {
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
    let block = Block::default()
        .title(format!("TODOs | {}", app.day.date))
        .borders(Borders::ALL).style(Style::default().fg(if active { Color::Yellow } else { Color::White }));
    let list = List::new(get_todos_list(&app, active)).block(block);
    f.render_widget(list, chunks[0]);
    let block = Block::default()
        .title("Notes")
        .borders(Borders::ALL);
    f.render_widget(block, chunks[1]);
}

pub fn stats_screen<B: Backend>(app: &App, f: &mut Frame<B>) {
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
