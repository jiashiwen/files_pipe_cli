use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::palette::tailwind,
    symbols,
    widgets::{Block, Padding, Paragraph, Widget},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NewTab {
    row_index: usize,
}

impl Widget for NewTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // RgbSwatch.render(area, buf);
        // let area = area.inner(Margin {
        //     vertical: 1,
        //     horizontal: 2,
        // });
        // Clear.render(area, buf);
        // Block::new()
        //     .title("Ratatouille Recipe".bold().white())
        //     .title_alignment(Alignment::Center)
        //     .style(THEME.content)
        //     .padding(Padding::new(1, 1, 2, 1))
        //     .render(area, buf);

        let scrollbar_area = Rect {
            y: area.y + 2,
            height: area.height - 3,
            ..area
        };
        // render_scrollbar(self.row_index, scrollbar_area, buf);

        // let area = area.inner(Margin {
        //     horizontal: 2,
        //     vertical: 1,
        // });
        // let [recipe, ingredients] =
        //     Layout::horizontal([Constraint::Length(44), Constraint::Min(0)]).areas(area);

        Paragraph::new("Welcome to the Ratatui tabs example!")
            .block(self.block())
            .render(area, buf);
    }
}

impl NewTab {
    fn block(self) -> Block<'static> {
        Block::bordered()
            // .border_set(symbols::border::PROPORTIONAL_TALL)
            .border_set(symbols::border::DOUBLE)
            .padding(Padding::horizontal(1))
            .border_style(self.palette().c700)
    }

    const fn palette(self) -> tailwind::Palette {
        tailwind::GREEN
    }
}
