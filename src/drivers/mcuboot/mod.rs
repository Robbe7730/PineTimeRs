mod header;
mod footer;

use header::MCUBootHeader;
use footer::MCUBootFooter;

use crate::drivers::flash::InternalFlash;

#[derive(Debug)]
pub struct MCUBoot {
    pub header: MCUBootHeader,
    pub footer: MCUBootFooter,
}

impl MCUBoot {
    pub fn get(internal_flash: &mut InternalFlash) -> Self {
        MCUBoot {
            header: MCUBootHeader::get(internal_flash),
            footer: MCUBootFooter::get(internal_flash),
        }
    }

    pub fn mark_valid(&mut self, internal_flash: &mut InternalFlash) {
        self.footer.is_valid = true;
        self.footer.write(internal_flash);
    }
}
