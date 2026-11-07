use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::Serialize;
use windows::{
    Media::Control::GlobalSystemMediaTransportControlsSessionManager,
    Storage::Streams::DataReader,
};

const MAX_ARTWORK_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NowPlayingArtwork {
    pub title: String,
    pub artist: String,
    pub artwork: Option<String>,
}

fn media_error(error: windows::core::Error) -> String {
    error.message().to_string()
}

pub fn get_now_playing_artwork() -> Result<Option<NowPlayingArtwork>, String> {
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .map_err(media_error)?
        .get()
        .map_err(media_error)?;

    // Liner is an Electron player and may remain registered next to another
    // music client. Prefer it when it advertises a Windows media session, then
    // retain the normal system-selected player behaviour for every other app.
    let liner_session = manager.GetSessions().ok().and_then(|sessions| {
        (0..sessions.Size().ok()?).find_map(|index| {
            let session = sessions.GetAt(index).ok()?;
            let source = session.SourceAppUserModelId().ok()?.to_string_lossy();
            source.to_ascii_lowercase().contains("liner").then_some(session)
        })
    });
    let session = match liner_session.or_else(|| manager.GetCurrentSession().ok()) {
        Some(session) => session,
        None => return Ok(None),
    };
    let properties = match session.TryGetMediaPropertiesAsync() {
        Ok(operation) => match operation.get() {
            Ok(properties) => properties,
            Err(_) => return Ok(None),
        },
        Err(_) => return Ok(None),
    };

    let title = properties.Title().map_err(media_error)?.to_string_lossy();
    let artist = properties.Artist().map_err(media_error)?.to_string_lossy();
    let thumbnail = match properties.Thumbnail() {
        Ok(thumbnail) => thumbnail,
        Err(_) => {
            return Ok(Some(NowPlayingArtwork {
                title,
                artist,
                artwork: None,
            }))
        }
    };
    let stream = match thumbnail.OpenReadAsync() {
        Ok(operation) => match operation.get() {
            Ok(stream) => stream,
            Err(_) => {
                return Ok(Some(NowPlayingArtwork {
                    title,
                    artist,
                    artwork: None,
                }))
            }
        },
        Err(_) => {
            return Ok(Some(NowPlayingArtwork {
                title,
                artist,
                artwork: None,
            }))
        }
    };
    let size = stream.Size().map_err(media_error)?;
    if size == 0 || size > MAX_ARTWORK_BYTES {
        return Ok(Some(NowPlayingArtwork {
            title,
            artist,
            artwork: None,
        }));
    }

    let reader = DataReader::CreateDataReader(&stream).map_err(media_error)?;
    reader.LoadAsync(size as u32).map_err(media_error)?.get().map_err(media_error)?;
    let mut bytes = vec![0; size as usize];
    reader.ReadBytes(&mut bytes).map_err(media_error)?;
    let content_type = stream.ContentType().map_err(media_error)?.to_string_lossy();
    let mime = if content_type.starts_with("image/") {
        content_type
    } else {
        "image/jpeg".to_owned()
    };

    Ok(Some(NowPlayingArtwork {
        title,
        artist,
        artwork: Some(format!("data:{mime};base64,{}", STANDARD.encode(bytes))),
    }))
}
