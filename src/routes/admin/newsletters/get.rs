use actix_web::{HttpResponse, http::header::ContentType};
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;

pub async fn get_newsletters_page(
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let mut msg_html = String::new();
    for m in flash_messages.iter() {
        writeln!(msg_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }
    let script = include_str!("./disable-submit-button.js");
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta http-equiv="content-type" content="text/html; charset=utf-8">
                <title>Admin dashboard</title>
            </head>
            <body>
                {msg_html}
                <h1>Publish a newsletter:</h1>
				<form id="publishForm" action="/admin/newsletters" method="post">
				<label
					>Title
					<input
					type="text"
					placeholder="Something enticing..."
					name="title"
                    required
					/>
				</label>
				<br />
				<label
					>Content (Text)
					<textarea
                    placeholder="Enter the content in plain text"
					rows="20"
                    cols="50"
					name="text_content"
                    required
                    ></textarea>
				</label>
				<br />
				<label
					>Content (HTML)
					<textarea
                    placeholder="Enter the content in HTML format"
					rows="20"
                    cols="50"
					name="html_content"
                    required
                    ></textarea>
				</label>
				<br />
				<button id="submitButton" type="submit">Publish</button>
				</form>
				<p><a href="/admin/dashboard">&lt;- Back</a></p>
            </body>
            <script>
            {script}
            </script>
            </html>
            "#
        )))
}
