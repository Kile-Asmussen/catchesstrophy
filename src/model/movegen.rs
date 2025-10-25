use crate::model::{BitBoard, Legal, PseudoLegal};

impl BitBoard {
    fn list_pseudomoves(self, _buffer: &mut Vec<PseudoLegal>) {}

    fn list_moves(self, _buffer: &mut Vec<Legal>) {}

    fn bless(_mv: PseudoLegal) -> Option<Legal> {
        None
    }
}
