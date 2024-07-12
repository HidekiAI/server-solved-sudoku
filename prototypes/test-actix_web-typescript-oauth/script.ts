// Note: Some sensitive data such as CLIENT_ID will be read from
// environment variables and swapped in during the build process. (build.js)
document.getElementById("login-btn")?.addEventListener("click", () => {
    const authUrl = 'https://accounts.google.com/o/oauth2/v2/auth?' +
        'response_type=code&' +
        'scope=openid%20email%20profile&' +
        'prompt=consent%20select_account&' +
        'state=state_parameter_passthrough_value&' +
        'redirect_uri=GOOGLE_REDIRECT_URI_SWAPPED_IN_BUILD&' +
        'client_id=GOOGLE_CLIENT_ID_FROM_SWAPPED_IN_BUILD' ;
    
    window.open(authUrl, '_blank', 'width=500,height=600');
});
