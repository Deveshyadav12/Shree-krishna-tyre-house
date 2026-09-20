use axum::{
    extract::Form,
    http::StatusCode,
    response::{Html, Redirect},
};

use serde::Deserialize;

use crate::pages::layout::page;

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

pub async fn login() -> Html<String> {
    let content = r#"

<div class="admin-login-page">

    <div class="admin-login-card">

        <div class="admin-login-brand">

            <div class="admin-login-logo">
                SK
            </div>

            <div>
                <strong>SHRI KRISHNA</strong>
                <span>TYRE HOUSE</span>
            </div>

        </div>


        <div class="admin-login-heading">

            <span class="admin-eyebrow">
                ADMIN PANEL
            </span>

            <h1>Welcome Back</h1>

            <p>
                Sign in to manage your tyre shop.
            </p>

        </div>


        <form
            class="admin-login-form"
            method="post"
            action="/admin/login"
        >

            <label for="username">
                Username
            </label>

            <input
                id="username"
                name="username"
                type="text"
                placeholder="Enter username"
                autocomplete="username"
                required
            >


            <label for="password">
                Password
            </label>

            <input
                id="password"
                name="password"
                type="password"
                placeholder="Enter password"
                autocomplete="current-password"
                required
            >


            <button
                type="submit"
                class="admin-login-button"
            >
                Sign In
            </button>

        </form>


        <div class="demo-login-box">

            <div class="demo-login-title">
                DEMO LOGIN
            </div>

            <div class="demo-login-row">
                <span>Username</span>
                <strong>admin</strong>
            </div>

            <div class="demo-login-row">
                <span>Password</span>
                <strong>admin123</strong>
            </div>

        </div>


        <a
            href="/"
            class="admin-back-link"
        >
            ← Back to Website
        </a>

    </div>

</div>

"#;

    Html(page("Admin Login", content))
}

pub async fn login_submit(
    Form(form): Form<LoginForm>,
) -> Result<Redirect, (StatusCode, Html<String>)> {
    if form.username == "admin" && form.password == "admin123" {
        return Ok(Redirect::to("/admin"));
    }

    let content = r#"

<div class="admin-login-page">

    <div class="admin-login-card">

        <div class="admin-login-brand">

            <div class="admin-login-logo">
                SK
            </div>

            <div>
                <strong>SHRI KRISHNA</strong>
                <span>TYRE HOUSE</span>
            </div>

        </div>


        <div class="admin-login-heading">

            <span class="admin-eyebrow">
                ADMIN PANEL
            </span>

            <h1>Login Failed</h1>

            <p>
                The username or password is incorrect.
            </p>

        </div>


        <div class="login-error-box">
            Invalid username or password.
        </div>


        <a
            href="/admin/login"
            class="admin-login-button login-retry-button"
        >
            Try Again
        </a>


        <div class="demo-login-box">

            <div class="demo-login-title">
                DEMO LOGIN
            </div>

            <div class="demo-login-row">
                <span>Username</span>
                <strong>admin</strong>
            </div>

            <div class="demo-login-row">
                <span>Password</span>
                <strong>admin123</strong>
            </div>

        </div>

    </div>

</div>

"#;

    Err((
        StatusCode::UNAUTHORIZED,
        Html(page("Login Failed", content)),
    ))
}
