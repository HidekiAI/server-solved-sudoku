#!/bin/bash
set -o nounset      # treat unset vars as errors
set -o errexit      # exit on command failure
set -o pipefail     # capture fail exit codes in piped commands

# Test client auth using cURL and zenity
# Make sure to setup `.env` file with actual data
_ZENITY=$(which zenity)
if [ -z "${_ZENITY}" ]; then
    echo "Please install zenity"
    exit 1
fi

# Load environment variables
source ../.env

function check_env_var() {
    if [ -z "${!1}" ]; then
        echo "Please set the ${1} environment variable"
        exit 1
    fi
}

# Sample request (HTTP POST):
#       https://accounts.google.com/o/oauth2/v2/auth?
#        response_type=code&
#        client_id=424911365001.apps.googleusercontent.com&
#        scope=openid%20email&
#        redirect_uri=https%3A//oauth2.example.com/code&
#        state=security_token%3D138r5719ru3e1%26url%3Dhttps%3A%2F%2Foauth2-login-demo.example.com%2FmyHome&
#        login_hint=jsmith@example.com&
#        nonce=0394852-3190485-2490358&
#        hd=example.com
function get_auth_url() {
    local CLIENT_ID="${1}"
    local REDIRECT_URI="${2}"
    local SCOPE="${3}"
    local STATE="${4}"
    echo "https://accounts.google.com/o/oauth2/v2/auth?response_type=code&client_id=${CLIENT_ID}&redirect_uri=${REDIRECT_URI}&scope=${SCOPE}&state=${STATE}"
}

function get_token_url() {
    echo "https://oauth2.googleapis.com/token"
}

function get_auth_code() {
    ${_ZENITY} --entry --text="Enter the authorization code:" --title="OAuth2 Authorization Code"
}

# authorization request is HTTP GET
# we'll start a temp HTTP server to listen for the auth code, and shut it down soon as we get the auth code
function request_auth_code() {
    local CLIENT_ID="${1}"
    local REDIRECT_URI="${2}"
    local SCOPE="${3}"

    # Generate authorization URL and prompt user
    AUTH_URL=$( get_auth_url "${CLIENT_ID}" "${REDIRECT_URI}" "${SCOPE}" "$(make_state_token)" )

    echo "# Please start the HTTPS redirect_url server listening for callback at address ${REDIRECT_URI}"
    read -n 1 -s -r -p "Press any key to continue"
    echo

    # curl to get the auth code, returns either "code=AUTH_CODE" or "error=access_denied"
    echo "# Connecting to ${AUTH_URL}"
    echo
    curl -k -X GET "${AUTH_URL}"
}

# token request is HTTP POST
function exchange_auth_code() {
    local CLIENT_ID="${1}"
    local CLIENT_SECRET="${2}"
    local AUTH_CODE="${3}"
    local REDIRECT_URI="${4}"
    local TOKEN_URL=$(get_token_url)
    curl -k -X POST "${TOKEN_URL}" \
        -d "client_id=${CLIENT_ID}" \
        -d "client_secret=${CLIENT_SECRET}" \
        -d "code=${AUTH_CODE}" \
        -d "redirect_uri=${REDIRECT_URI}" \
        -d "grant_type=authorization_code"
}

function make_state_token() {
    # Python example:
    # # Create a state token to prevent request forgery.
    # # Store it in the session for later validation.
    # state = hashlib.sha256(os.urandom(1024)).hexdigest()
    # session['state'] = state
    echo $(head -c 32 /dev/urandom | base64 | tr -d '+/=')
}

#function make_HTML_header_with_tokens(){
#    # Python example:
#    # # Set the client ID, token state, and application name in the HTML while
#    # # serving it.
#    # response = make_response(
#    #     render_template('index.html',
#    #                     CLIENT_ID=CLIENT_ID,
#    #                     STATE=state,
#    #                     APPLICATION_NAME=APPLICATION_NAME))
#    STATE_TOKEN=$(make_state_token)
#    echo "<html><head><title>OAuth2 Authorization</title></head><body>"
#    echo "<h1>OAuth2 Authorization</h1>"
#    echo "<p>Please open your web browser and go to the following URL to authorize the application:</p>"
#    echo "<p><a href=\"$(get_auth_url ${GOOGLE_CLIENT_ID} ${GOOGLE_REDIRECT_URI} ${SCOPE} ${STATE_TOKEN})\">Authorize</a></p>"
#    echo "<p>After authorizing the application, enter the authorization code below:</p>"
#    echo "<form action=\"/auth\" method=\"post\">"
#    echo "<input type=\"text\" name=\"auth_code\" placeholder=\"Authorization Code\">"
#    echo "<input type=\"hidden\" name=\"state\" value=\"${STATE_TOKEN}\">"
#    echo "<input type=\"submit\" value=\"Submit\">"
#    echo "</form>"
#    echo "</body></html>"
#}

# OAuth2 parameters
if [ -z "${GOOGLE_CLIENT_ID}" ]; then
    echo "Please set the GOOGLE_CLIENT_ID environment variable"
    CLIENT_ID="$1"
else
    CLIENT_ID="${GOOGLE_CLIENT_ID}"
fi
if [ -z "${GOOGLE_CLIENT_SECRET}" ]; then
    echo "Please set the GOOGLE_CLIENT_SECRET environment variable "
    CLIENT_SECRET="$2"
else
    CLIENT_SECRET="${GOOGLE_CLIENT_SECRET}"
fi
if [ -z "${GOOGLE_REDIRECT_URI}" ]; then
    echo "Please set the GOOGLE_REDIRECT_URI environment variable"
    REDIRECT_URI="$3"
else
    REDIRECT_URI="${GOOGLE_REDIRECT_URI}"
fi
check_env_var "CLIENT_ID"
check_env_var "CLIENT_SECRET"
check_env_var "REDIRECT_URI"

SCOPE="openid profile email"

# curl to get the auth code
${_ZENITY} --info --text="Please open your web browser and go to the following URL to authorize the application:" --title="OAuth2 Authorization"

# Prompt user to enter authorization code
request_auth_code ${CLIENT_ID} ${REDIRECT_URI} ${SCOPE}
AUTH_CODE=$(get_auth_code)
check_env_var "AUTH_CODE"

# Exchange authorization code for tokens
exchange_auth_code ${CLIENT_ID} ${CLIENT_SECRET} ${AUTH_CODE} ${REDIRECT_URI}