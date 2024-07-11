document.getElementById("login-btn")?.addEventListener("click", () => {
    const authUrl = 'https://accounts.google.com/o/oauth2/v2/auth?' +
        'scope=email%20profile&' +
        'include_granted_scopes=true&' +
        'response_type=token&' +
        'state=state_parameter_passthrough_value&' +
        'redirect_uri=http://localhost:8080/oauth2/callback&' +
        'client_id=YOUR_CLIENT_ID';
    
    window.open(authUrl, '_blank', 'width=500,height=600');
});
