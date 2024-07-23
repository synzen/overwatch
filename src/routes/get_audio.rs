use crate::utils::app_error::AppError;
use axum::{
    body::Body,
    response::{AppendHeaders, IntoResponse, Response},
};
use tokio::fs::File;

pub async fn get_audio() -> Result<Response, AppError> {
    let path = "src/media/two_minutes.wav";
    let audio = File::open(path).await.unwrap();

    let stream = tokio_util::io::ReaderStream::new(audio);
    // let body = BodyDataStream(stream);
    let response_body = Body::from_stream(stream);

    let headers = AppendHeaders([
        ("content-type", "audio/wav"),
        ("content-disposition", "inline; filename=\"audio.wav\""),
    ]);

    Ok((headers, response_body).into_response())
}
