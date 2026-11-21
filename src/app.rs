//! Keys -> core, and drawing: a parameter menu, a preset gallery and the
//! interference bitmap drawn as a handful of merged rectangles.

use gam::menu::{PixelColor, Point};
use gam::GlyphStyle;
use moire_core::model::{Model, Outcome, Screen as View};
use precursor_appkit::keys::Key;
use precursor_appkit::screen::Screen;

#[derive(PartialEq, Eq)]
pub enum Action {
    Stay,
    Leave,
}

pub struct App {
    screen: Screen,
    model: Model,
}

impl App {
    pub fn new(screen: Screen) -> Self { Self { screen, model: Model::new() } }

    pub fn screen(&self) -> &Screen { &self.screen }

    pub fn on_key(&mut self, key: Key) -> Action {
        match self.model.on_key(key) {
            Outcome::Leave => Action::Leave,
            Outcome::Stay => Action::Stay,
        }
    }

    pub fn on_foreground(&mut self) {
        self.screen.refresh_size();
        self.draw();
    }

    pub fn draw(&mut self) {
        self.screen.clear();
        match self.model.screen {
            View::Menu => self.draw_menu(),
            View::Preview => self.draw_preview(),
            View::Gallery => self.draw_gallery(),
        }
        self.screen.flush();
    }

    fn draw_menu(&mut self) {
        let top = self.screen.header("Moire Generator");
        let bottom = self.screen.footer("↑↓ field · ←→ change · ⏎ select · ⌫ back");
        let labels = self.model.menu_labels();
        let rows = self.screen.list(top, bottom, &labels, &self.model.menu);
        self.model.set_visible(rows);
    }

    fn draw_gallery(&mut self) {
        let top = self.screen.header("Gallery");
        let bottom = self.screen.footer("↑↓ browse · ⏎ show · ⌫ back");
        let labels = self.model.gallery_labels();
        let rows = self.screen.list(top, bottom, &labels, &self.model.gallery);
        self.model.set_visible(rows);
    }

    fn draw_preview(&mut self) {
        let size = self.screen.size();
        let w = size.x as i32;
        // Same geometry as Screen::header / Screen::footer.
        let top = self.screen.line_height(GlyphStyle::Bold) + 8;
        let bottom = size.y - self.screen.line_height(GlyphStyle::Small) - 6;
        let h = (bottom - top).max(1) as i32;
        let rects = self.model.rects(w, h);
        self.screen.fill_rects(
            rects.into_iter().map(|(x0, y0, x1, y1)| {
                (Point::new(x0 as isize, y0 as isize + top), Point::new(x1 as isize, y1 as isize + top))
            }),
            PixelColor::Dark,
        );
        self.screen.header(&self.model.title());
        self.screen.footer("←→ B angle · ↑↓ B spacing · ⌫ back");
    }
}
