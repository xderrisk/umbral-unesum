use gettextrs::gettext;

pub async fn register_camera(
    mac: &str,
    classroom_name: &str,
    api_key: &str,
) -> Result<String, String> {
    if api_key.is_empty() {
        return Err(gettext(
            "Firebase API Key is not configured in the application settings.",
        ));
    }

    let client = reqwest::Client::new();
    let url_auth = format!(
        "https://identitytoolkit.googleapis.com/v1/accounts:signUp?key={}",
        api_key
    );

    let mock_email = format!("camera_{}@umbral.unesum.edu", mac);
    let mock_password = format!("Umbral.{}#", mac);

    let body_auth = serde_json::json!({
        "email": mock_email,
        "password": mock_password,
        "returnSecureToken": true
    });

    let response_auth = client
        .post(&url_auth)
        .json(&body_auth)
        .send()
        .await
        .map_err(|e| {
            format!(
                "{}: {}",
                gettext("Connection error during authentication"),
                e
            )
        })?;

    let (uid, id_token) = if response_auth.status().is_success() {
        let json_res: serde_json::Value = response_auth.json().await.map_err(|e| e.to_string())?;
        let local_id = json_res
            .get("localId")
            .and_then(|id| id.as_str())
            .unwrap_or("")
            .to_string();
        let token = json_res
            .get("idToken")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
        (local_id, token)
    } else {
        let error_text = response_auth.text().await.unwrap_or_default();
        if error_text.contains("EMAIL_EXISTS") {
            return Err(gettext(
                "The device is already authenticated. Delete the user in the Firebase console to retry.",
            ));
        }
        return Err(format!(
            "{}: {}",
            gettext("Authentication error"),
            error_text
        ));
    };

    if uid.is_empty() || id_token.is_empty() {
        return Err(gettext("Failed to obtain credentials from Firebase Auth"));
    }

    let url_db = format!(
        "https://myapplication-65c31ca7-default-rtdb.firebaseio.com/cameras/{}.json?auth={}",
        uid, id_token
    );

    let body_db = serde_json::json!({
        "name": classroom_name,
    });

    let response_db = client
        .put(&url_db)
        .json(&body_db)
        .send()
        .await
        .map_err(|e| {
            format!(
                "{}: {}",
                gettext("Connection error during database write"),
                e
            )
        })?;

    if response_db.status().is_success() {
        Ok(uid)
    } else {
        let err_db = response_db.text().await.unwrap_or_default();
        Err(format!(
            "{}: {}",
            gettext("Authentication successful, but database rules rejected the write operation"),
            err_db
        ))
    }
}

pub async fn delete_camera(uid: &str, id_token: &str, api_key: &str) -> Result<(), String> {
    if api_key.is_empty() || uid.is_empty() || id_token.is_empty() {
        return Err(gettext("Missing required parameters for deletion."));
    }

    let client = reqwest::Client::new();

    let url_db = format!(
        "https://myapplication-65c31ca7-default-rtdb.firebaseio.com/cameras/{}.json?auth={}",
        uid, id_token
    );

    let response_db = client.delete(&url_db).send().await.map_err(|e| {
        format!(
            "{}: {}",
            gettext("Connection error during database deletion"),
            e
        )
    })?;

    if !response_db.status().is_success() {
        let err_db = response_db.text().await.unwrap_or_default();
        return Err(format!(
            "{}: {}",
            gettext("Failed to delete database record"),
            err_db
        ));
    }

    let url_auth = format!(
        "https://identitytoolkit.googleapis.com/v1/accounts:delete?key={}",
        api_key
    );

    let body_auth = serde_json::json!({
        "idToken": id_token
    });

    let response_auth = client
        .post(&url_auth)
        .json(&body_auth)
        .send()
        .await
        .map_err(|e| {
            format!(
                "{}: {}",
                gettext("Connection error during account deletion"),
                e
            )
        })?;

    if response_auth.status().is_success() {
        Ok(())
    } else {
        let error_text = response_auth.text().await.unwrap_or_default();
        Err(format!(
            "{}: {}",
            gettext("Error deleting Firebase account"),
            error_text
        ))
    }
}

pub async fn update_camera_name(uid: &str, id_token: &str, name: &str) -> Result<(), String> {
    if uid.is_empty() || id_token.is_empty() {
        return Err(gettext("Missing required parameters for update."));
    }

    let client = reqwest::Client::new();
    let url_db = format!(
        "https://myapplication-65c31ca7-default-rtdb.firebaseio.com/cameras/{}.json?auth={}",
        uid, id_token
    );

    let body_db = serde_json::json!({ "name": name });

    let response_db = client
        .patch(&url_db)
        .json(&body_db)
        .send()
        .await
        .map_err(|e| {
            format!(
                "{}: {}",
                gettext("Connection error during database update"),
                e
            )
        })?;

    if response_db.status().is_success() {
        Ok(())
    } else {
        let err_db = response_db.text().await.unwrap_or_default();
        Err(format!(
            "{}: {}",
            gettext("Failed to update database record"),
            err_db
        ))
    }
}

pub async fn login_camera(mac: &str, api_key: &str) -> Result<(String, String), String> {
    if api_key.is_empty() {
        return Err(gettext("Firebase API Key is not configured."));
    }

    let client = reqwest::Client::new();
    let url_login = format!(
        "https://identitytoolkit.googleapis.com/v1/accounts:signInWithPassword?key={}",
        api_key
    );

    let mock_email = format!("camera_{}@umbral.unesum.edu", mac);
    let mock_password = format!("Umbral.{}#", mac);

    let body_login = serde_json::json!({
        "email": mock_email,
        "password": mock_password,
        "returnSecureToken": true
    });

    let response = client
        .post(&url_login)
        .json(&body_login)
        .send()
        .await
        .map_err(|e| format!("{}: {}", gettext("Connection error during camera login"), e))?;

    if response.status().is_success() {
        let json_res: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

        let uid = json_res
            .get("localId")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let id_token = json_res
            .get("idToken")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if uid.is_empty() || id_token.is_empty() {
            return Err(gettext("Failed to parse credentials from login response."));
        }

        Ok((uid, id_token))
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(format!(
            "{}: {}",
            gettext("Authentication failed for camera deletion"),
            error_text
        ))
    }
}
