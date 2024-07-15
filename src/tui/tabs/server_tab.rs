use std::sync::{Arc, RwLock};

use dashmap::DashMap;
use itertools::Itertools;
use once_cell::sync::Lazy;
use palette::{cast::ArraysInto, encoding::gamma};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{palette::tailwind, Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Text},
    widgets::{
        Block, Cell, Clear, HighlightSpacing, Padding, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, StatefulWidget, Table, TableState, Widget, Wrap,
    },
};

use crate::{
    request::TaskServer,
    resources::list_servers_from_cf,
    tui::{colors::RgbSwatch, THEME},
};

// use crate::{RgbSwatch, THEME};

const ITEM_HEIGHT: usize = 4;
const PALETTES: [tailwind::Palette; 4] = [
    tailwind::BLUE,
    tailwind::EMERALD,
    tailwind::INDIGO,
    tailwind::RED,
];

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct TableColors {
    buffer_bg: Color,
    header_bg: Color,
    header_fg: Color,
    row_fg: Color,
    selected_style_fg: Color,
    normal_row_color: Color,
    alt_row_color: Color,
    footer_border_color: Color,
}

impl TableColors {
    const fn new(color: &tailwind::Palette) -> Self {
        Self {
            buffer_bg: tailwind::SLATE.c950,
            header_bg: color.c900,
            header_fg: tailwind::SLATE.c200,
            row_fg: tailwind::SLATE.c200,
            selected_style_fg: color.c400,
            normal_row_color: tailwind::SLATE.c950,
            alt_row_color: tailwind::SLATE.c900,
            footer_border_color: color.c400,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Data {
    id: String,
    name: String,
    url: String,
}

impl Data {
    const fn ref_array(&self) -> [&String; 3] {
        [&self.id, &self.name, &self.url]
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn url(&self) -> &str {
        &self.url
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Ingredient {
    quantity: &'static str,
    name: &'static str,
}

impl Ingredient {
    #[allow(clippy::cast_possible_truncation)]
    fn height(&self) -> u16 {
        self.name.lines().count() as u16
    }
}

impl<'a> From<Ingredient> for Row<'a> {
    fn from(i: Ingredient) -> Self {
        Row::new(vec![i.quantity, i.name]).height(i.height())
    }
}

// https://www.realsimple.com/food-recipes/browse-all-recipes/ratatouille
const RECIPE: &[(&str, &str)] = &[
    (
        "Step 1: ",
        "Over medium-low heat, add the oil to a large skillet with the onion, garlic, and bay \
        leaf, stirring occasionally, until the onion has softened.",
    ),
    (
        "Step 2: ",
        "Add the eggplant and cook, stirring occasionally, for 8 minutes or until the eggplant \
        has softened. Stir in the zucchini, red bell pepper, tomatoes, and salt, and cook over \
        medium heat, stirring occasionally, for 5 to 7 minutes or until the vegetables are \
        tender. Stir in the basil and few grinds of pepper to taste.",
    ),
];

const INGREDIENTS: &[Ingredient] = &[
    Ingredient {
        quantity: "4 tbsp",
        name: "olive oil",
    },
    Ingredient {
        quantity: "1",
        name: "onion thinly sliced",
    },
    Ingredient {
        quantity: "4",
        name: "cloves garlic\npeeled and sliced",
    },
    Ingredient {
        quantity: "1",
        name: "small bay leaf",
    },
    Ingredient {
        quantity: "1",
        name: "small eggplant cut\ninto 1/2 inch cubes",
    },
    Ingredient {
        quantity: "1",
        name: "small zucchini halved\nlengthwise and cut\ninto thin slices",
    },
    Ingredient {
        quantity: "1",
        name: "red bell pepper cut\ninto slivers",
    },
    Ingredient {
        quantity: "4",
        name: "plum tomatoes\ncoarsely chopped",
    },
    Ingredient {
        quantity: "1 tsp",
        name: "kosher salt",
    },
    Ingredient {
        quantity: "1/4 cup",
        name: "shredded fresh basil\nleaves",
    },
    Ingredient {
        quantity: "",
        name: "freshly ground black\npepper",
    },
];

pub static GLOBAL_SERVER_TABLE_DATA: Lazy<Arc<DashMap<String, TaskServer>>> = Lazy::new(|| {
    let server_table_data = Arc::new(DashMap::<String, TaskServer>::new());
    server_table_data
});

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ServerTab {
    row_index: usize,
    colors: TableColors,
    color_index: usize,
    pub longest_item_lens: (u16, u16, u16),
}

impl ServerTab {
    /// Select the previous item in the ingredients list (with wrap around)
    pub fn prev(&mut self) {
        self.flush_data();
        // self.row_index = self.row_index.saturating_add(INGREDIENTS.len() - 1) % INGREDIENTS.len();
        self.row_index = self
            .row_index
            .saturating_add(GLOBAL_SERVER_TABLE_DATA.len() - 1)
            % GLOBAL_SERVER_TABLE_DATA.len();
    }

    /// Select the next item in the ingredients list (with wrap around)
    pub fn next(&mut self) {
        self.flush_data();
        // self.row_index = self.row_index.saturating_add(1) % INGREDIENTS.len();
        self.row_index = self.row_index.saturating_add(1) % GLOBAL_SERVER_TABLE_DATA.len();
    }

    pub fn set_colors(&mut self) {
        self.colors = TableColors::new(&PALETTES[self.color_index]);
    }

    pub fn flush_data(&mut self) {
        let vec_taskserver = match list_servers_from_cf() {
            Ok(v) => v,
            Err(e) => {
                log::error!("{:?}", e);
                let v = Vec::<(String, TaskServer)>::new();
                v
            }
        };
        GLOBAL_SERVER_TABLE_DATA.clear();

        let mut id_len = 0;
        let mut name_len = 0;
        let mut url_len = 0;
        for (id, task_server) in vec_taskserver {
            if id.len().gt(&id_len) {
                id_len = id.len()
            }

            if task_server.name.len().gt(&name_len) {
                name_len = task_server.name.len()
            }

            if task_server.url.len().gt(&url_len) {
                url_len = task_server.url.len()
            }
            GLOBAL_SERVER_TABLE_DATA.insert(id, task_server);
            GLOBAL_SERVER_TABLE_DATA.shrink_to_fit();
        }
        let id_len_u16 = id_len.try_into().unwrap();
        let name_len_u16 = name_len.try_into().unwrap();
        let url_len_u16 = url_len.try_into().unwrap();
        self.longest_item_lens = (id_len_u16, name_len_u16, url_len_u16)
    }
}

impl Widget for ServerTab {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        self.set_colors();
        // RgbSwatch.render(area, buf);
        let area = area.inner(Margin {
            vertical: 1,
            horizontal: 2,
        });
        Clear.render(area, buf);
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
        render_scrollbar(self.row_index, scrollbar_area, buf);

        // let area = area.inner(Margin {
        //     horizontal: 2,
        //     vertical: 1,
        // });
        // let [recipe, ingredients] =
        //     Layout::horizontal([Constraint::Length(44), Constraint::Min(0)]).areas(area);

        // render_recipe(recipe, buf);
        render_server_table(&self, area, buf);
    }
}

fn render_server_table(server_tab: &ServerTab, area: Rect, buf: &mut Buffer) {
    let mut state = TableState::default().with_selected(Some(server_tab.row_index));
    // let rows = INGREDIENTS.iter().copied();
    let theme = THEME.recipe;

    let header_style = Style::default()
        .fg(server_tab.colors.header_fg)
        .bg(server_tab.colors.header_bg);
    let selected_style = Style::default()
        .add_modifier(Modifier::REVERSED)
        .fg(server_tab.colors.selected_style_fg);

    let header = ["Id", "Name", "Url"]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(header_style)
        .height(1);
    let data_iter = GLOBAL_SERVER_TABLE_DATA.iter();

    let rows = data_iter.enumerate().map(|(i, data)| {
        let color = match i % 2 {
            0 => server_tab.colors.normal_row_color,
            _ => server_tab.colors.alt_row_color,
        };

        let item = vec![
            data.key().to_string(),
            data.value().name.clone(),
            data.value().url.clone(),
        ];
        item.into_iter()
            .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
            .collect::<Row>()
            .style(Style::new().fg(server_tab.colors.row_fg).bg(color))
            .height(4)
    });
    let bar = " █ ";
    let table = Table::new(
        rows,
        [
            // + 1 is for padding.
            Constraint::Length(server_tab.longest_item_lens.0 + 1),
            Constraint::Min(server_tab.longest_item_lens.1 + 1),
            Constraint::Min(server_tab.longest_item_lens.2),
        ],
    )
    .header(header)
    .highlight_style(selected_style)
    .highlight_symbol(Text::from(vec![
        "".into(),
        bar.into(),
        bar.into(),
        "".into(),
    ]))
    .bg(server_tab.colors.buffer_bg)
    .highlight_spacing(HighlightSpacing::Always);

    StatefulWidget::render(
        // Table::new(rows, [Constraint::Length(7), Constraint::Length(30)])
        //     .block(Block::new().style(theme.ingredients))
        //     // .header(Row::new(vec!["Qty", "Ingredient"]).style(theme.ingredients_header))
        //     .header(Row::new(vec!["Id", "Name", "Url"]).style(theme.ingredients_header))
        //     .highlight_style(Style::new().light_yellow()),
        table, area, buf, &mut state,
    );
}

fn render_scrollbar(position: usize, area: Rect, buf: &mut Buffer) {
    let mut state = ScrollbarState::default()
        .content_length(INGREDIENTS.len())
        .viewport_content_length(6)
        .position(position);
    Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(None)
        .end_symbol(None)
        .track_symbol(None)
        .thumb_symbol("▐")
        .render(area, buf, &mut state);
}

fn generate_fake_names() -> Vec<Data> {
    use fakeit::{address, contact, name};

    (0..20)
        .map(|_| {
            let name = name::full();
            let address = format!(
                "{}\n{}, {} {}",
                address::street(),
                address::city(),
                address::state(),
                address::zip()
            );
            let email = contact::email();

            Data {
                id: name,
                name: address,
                url: email,
            }
        })
        .sorted_by(|a, b| a.id.cmp(&b.id))
        .collect_vec()
}
