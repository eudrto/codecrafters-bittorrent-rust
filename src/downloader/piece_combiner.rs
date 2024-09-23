use std::{io::SeekFrom, path::Path};

use tokio::{
    fs::File,
    io::{AsyncSeekExt, AsyncWriteExt},
    sync::mpsc::UnboundedReceiver,
};

use super::parts::PieceResp;

pub async fn piece_combiner(
    mut piece_receiver: UnboundedReceiver<PieceResp>,
    piece_size: u32,
    path: impl AsRef<Path>,
) {
    let mut file = File::create(&path).await.unwrap();
    loop {
        let Some(piece) = piece_receiver.recv().await else {
            return;
        };

        let seek_from = SeekFrom::Start(piece.idx as u64 * piece_size as u64);
        file.seek(seek_from).await.unwrap();
        file.write(&piece.bytes).await.unwrap();
    }
}
