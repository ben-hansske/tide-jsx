use pretty_assertions::assert_eq;
use std::borrow::Cow;
use tide::StatusCode;
use tide_jsx::html::HTML5Doctype;
use tide_jsx::{component, html, raw, rsx, view, Render};

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/fail/*.rs");
}

#[test]
fn works_with_dashes() {
    use pretty_assertions::assert_eq;

    let value = html! { <div data-id={"myid"} /> };
    assert_eq!(value, r#"<div data-id="myid"/>"#);
}

#[test]
fn works_with_raw() {
    let actual = html! {
        <div>{raw!("<Hello />")}</div>
    };

    assert_eq!(actual, "<div><Hello /></div>");
}

#[test]
fn works_with_raw_ident() {
    let actual = html! {
        <input r#type={"text"} />
    };

    assert_eq!(actual, r#"<input type="text"/>"#);
}

#[test]
fn works_with_keywords() {
    assert_eq!(html! { <input type={"text"} /> }, r#"<input type="text"/>"#);
    assert_eq!(html! { <label for={"me"} /> }, r#"<label for="me"/>"#);
}

#[test]
fn element_ordering() {
    let actual = html! {
      <ul>
        <li>{"1"}</li>
        <li>{"2"}</li>
        <li>{"3"}</li>
      </ul>
    };

    assert_eq!(actual, "<ul><li>1</li><li>2</li><li>3</li></ul>");

    let deep = html! {
      <div>
        <h1>{"A list"}</h1>
        <hr />
        <ul>
          <li>{"1"}</li>
          <li>{"2"}</li>
          <li>{"3"}</li>
        </ul>
      </div>
    };

    assert_eq!(
        deep,
        "<div><h1>A list</h1><hr/><ul><li>1</li><li>2</li><li>3</li></ul></div>"
    );
}

#[test]
fn some_none() {
    #[component]
    fn Answer(a: i8) {
        rsx! {
          <>
            {match a {
              42 => Some("Yes"),
              _ => None,
            }}
          </>
        }
    }

    assert_eq!(html! { <Answer a={42} /> }, "Yes");
    assert_eq!(html! { <Answer a={44} /> }, "");
}

#[test]
fn owned_string() {
    #[component]
    fn Welcome<'kind, 'name>(kind: &'kind str, name: &'name str) {
        rsx! {
            <h1 class={format!("{}-title", kind)}>
                {format!("Hello, {}", name)}
            </h1>
        }
    }

    assert_eq!(
        html! { <Welcome kind={"alien"} name={"Yoda"} /> },
        r#"<h1 class="alien-title">Hello, Yoda</h1>"#
    );
}

#[test]
fn cow_str() {
    let owned1 = "Borrowed from owned".to_owned();
    let owned2 = "Owned".to_owned();

    assert_eq!(
        html! {
            <div>
                <p>{Cow::Borrowed("Static")}</p>
                <p>{Cow::<'_, str>::Borrowed(&owned1)}</p>
                <p>{Cow::<'_, str>::Owned(owned2)}</p>
            </div>
        },
        r#"<div><p>Static</p><p>Borrowed from owned</p><p>Owned</p></div>"#,
    );
}

#[test]
fn number() {
    let num = 42;

    assert_eq!(html! { <p>{num}</p> }, "<p>42</p>")
}

#[test]
fn vec() {
    let list = vec!["Mouse", "Rat", "Hamster"];

    assert_eq!(
        html! {
            <ul>
                {
                    list
                        .into_iter()
                        .map(|text| rsx! { <li>{text}</li> })
                        .collect::<Vec<_>>()
                }
            </ul>
        },
        "<ul><li>Mouse</li><li>Rat</li><li>Hamster</li></ul>"
    )
}

#[async_std::test]
async fn render_view() -> std::io::Result<()> {
    let result = view! { <p>{"hello"}</p> } as tide::Result;
    let mut res = result.unwrap();
    assert_eq!(res.status(), StatusCode::Ok);
    assert_eq!(
        res.header("content-type").unwrap().as_str(),
        tide::http::mime::HTML.to_string()
    );
    assert_eq!(res.take_body().into_string().await.unwrap(), "<p>hello</p>");
    Ok(())
}

mod kaki {
    use crate::other::ExternalPage;
    use crate::{component, html, rsx, HTML5Doctype, Render};

    // This can be any layout we want
    #[component]
    fn Page<'a, Children: Render>(title: &'a str, children: Children) {
        rsx! {
          <>
            <HTML5Doctype />
            <html>
              <head><title>{title}</title></head>
              <body>
                {children}
              </body>
            </html>
          </>
        }
    }

    #[test]
    fn test() {
        let actual = html! {
          <Page title={"Home"}>
            {format!("Welcome, {}", "Gal")}
          </Page>
        };
        let expected = concat!(
            "<!DOCTYPE html>",
            "<html>",
            "<head><title>Home</title></head>",
            "<body>",
            "Welcome, Gal",
            "</body>",
            "</html>"
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn externals_test() {
        let actual = html! {
          <ExternalPage title={"Home"} subtitle={"Foo"}>
            {format!("Welcome, {}", "Gal")}
          </ExternalPage>
        };

        let expected = concat!(
            "<!DOCTYPE html>",
            "<html>",
            "<head><title>Home</title></head>",
            "<body>",
            "<h1>Foo</h1>",
            "Welcome, Gal",
            "</body>",
            "</html>"
        );
        assert_eq!(actual, expected);
    }
}

/// ## Other
///
/// Module for testing component visibility when imported from other modules.

mod other {
    use crate::{component, rsx, HTML5Doctype, Render};

    #[component]
    pub fn ExternalPage<'title, 'subtitle, Children: Render>(
        title: &'title str,
        subtitle: &'subtitle str,
        children: Children,
    ) {
        rsx! {
            <>
              <HTML5Doctype />
              <html>
                <head><title>{title}</title></head>
                <body>
                  <h1>{subtitle}</h1>
                  {children}
                </body>
              </html>
            </>
        }
    }
}

mod integration {
    use crate::view;
    use tide_testing::TideTestingExt;

    #[async_std::test]
    async fn test_server() -> tide::Result<()> {
        let mut app = tide::new();
        app.at("/").get(|_| async {
            view! {
                <div>
                  <p>{"Hello World"}</p>
                </div>
            }
        });

        assert_eq!(
            app.get("/").recv_string().await?,
            "<div><p>Hello World</p></div>"
        );
        assert_eq!(
            app.post("/missing").await?.status(),
            tide::http::StatusCode::NotFound
        );
        Ok(())
    }
}
