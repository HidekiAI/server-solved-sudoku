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

# See: https://developers.google.com/identity/openid-connect/openid-connect#obtaininguserprofileinformation on scope
# Sample request (HTTP POST):
#       https://accounts.google.com/o/oauth2/v2/auth?
#        response_type=code&
#        client_id=424911365001.apps.googleusercontent.com&
#        scope=openid%20profile%20email&
#        redirect_uri=https%3A//oauth2.example.com/code&
#        state=security_token%3D138r5719ru3e1%26url%3Dhttps%3A%2F%2Foauth2-login-demo.example.com%2FmyHome&
#        prompt=consent%20select_account&
#        login_hint=jsmith@example.com&
#        nonce=0394852-3190485-2490358&
#        hd=example.com
function get_auth_url() {
    local CLIENT_ID="${1}"
    local REDIRECT_URI="${2}"
    local SCOPE="${3}"
    local STATE="${4}"
    #echo "https://accounts.google.com/o/oauth2/v2/auth?response_type=code&client_id=${CLIENT_ID}&redirect_uri=${REDIRECT_URI}&scope=${SCOPE}&state=${STATE}&prompt=consent%20select_account"
    echo "https://accounts.google.com/o/oauth2/v2/auth?response_type=code&client_id=${CLIENT_ID}&redirect_uri=${REDIRECT_URI}&scope=${SCOPE}&prompt=consent%20select_account"
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
    # use netcat to listen to http://localhost:8080/
    nc -l -p 8080 > /tmp/auth_get_listener.log &

    # curl to get the auth code, returns either "code=AUTH_CODE" or "error=access_denied"
    echo
    echo "############################################################################################################"
    echo "# Open to your browser to the following URL to authorize the application:"
    echo
    echo "${AUTH_URL}"
    echo
    echo "############################################################################################################"
    echo
    curl --verbose -k -X GET "${AUTH_URL}"

    AUTH_URL_AMPER=$(echo "${AUTH_URL}" | sed 's/&/%26/g')
    _TEXT="Please open your web browser and go to the following URL to authorize the application:\n\n${AUTH_URL_AMPER}\n\nNOTE: DO NOT copy this one, copy-paste from the one on Console output..."
    echo $_TEXT
    ${_ZENITY} --info --text="$_TEXT" --title="OAuth2 Authorization"
}

function build_token_request_body() {
    local CLIENT_ID="${1}"
    local CLIENT_SECRET="${2}"
    local AUTH_CODE="${3}"
    local REDIRECT_URI="${4}"
    local GRANT_TYPE="${5}"
    # JSON body
    echo "{
        \"client_id\": \"${CLIENT_ID}\",
        \"client_secret\": \"${CLIENT_SECRET}\",
        \"code\": \"${AUTH_CODE}\",
        \"redirect_uri\": \"${REDIRECT_URI}\",
        \"grant_type\": \"${GRANT_TYPE}\"
    }"
}

# token request is HTTP POST
function exchange_auth_code() {
    local CLIENT_ID="${1}"
    local CLIENT_SECRET="${2}"
    local AUTH_CODE="${3}"
    local REDIRECT_URI="${4}"
    local TOKEN_URL=$(get_token_url)
    local JSON_BODY=$(build_token_request_body "${CLIENT_ID}" "${CLIENT_SECRET}" "${AUTH_CODE}" "${REDIRECT_URI}" "authorization_code")

    echo "# Requesting tokens from ${TOKEN_URL}" > /tmp/token_request.log
    echo "# JSON body:" >> /tmp/token_request.log
    echo "${JSON_BODY}" >> /tmp/token_request.log
    echo >> /tmp/token_request.log

    # Save response into a file
    curl --verbose -k -X POST "${TOKEN_URL}" \
        -H "Content-Type: application/json" \
        -d "${JSON_BODY}" -o "token_response.json"
    cat "token_response.json" >> /tmp/token_request.log

    # Extract access token
    ACCESS_TOKEN=$(jq -r '.access_token' "token_response.json")
    echo "Access Token: ${ACCESS_TOKEN}" >> /tmp/token_request.log
    echo ${ACCESS_TOKEN}
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

SCOPE="openid%20profile%20email"

# curl to get the auth code
echo "################################################## request_auth_code"
# Prompt user to enter authorization code
request_auth_code ${CLIENT_ID} ${REDIRECT_URI} ${SCOPE}
AUTH_CODE=$(get_auth_code)

# if /tmp/auth_get_listener.log has captured the redirect to localhost:8080, parse it and extract AUTH_CODE
# Sample capture:
#       GET /auth_callback?state=kM8JHEwKJurORMeYvSgIPE3Dz9noXlSbNsAnj5ZE1L4&code=4%2F0ATx3LY5LWFieCmErwAb8YRacDSc33FMlY2JHYdU83VtJf02_NYt4WtRtuPk_ZIjrInMceA&scope=openid&authuser=2&prompt=consent HTTP/1.1
#       Host: localhost:8080
# Assume 2nd '&' on first line is "code=AUTH_CODE"
_FIRST_LINE=$(head -n 1 /tmp/auth_get_listener.log)
AUTH_CODE_PARAM1=$(echo "${_FIRST_LINE}" | cut -d'&' -f1 )
AUTH_CODE_PARAM2=$(echo "${_FIRST_LINE}" | cut -d'&' -f2 )
echo -e "# Found parameter /tmp/auth_get_listener.log:\n\t'${AUTH_CODE_PARAM1}'\n\t'${AUTH_CODE_PARAM2}"
AUTH_CODE1=$(echo "${AUTH_CODE_PARAM1}" | grep "code=" | cut -d'=' -f2)
AUTH_CODE2=$(echo "${AUTH_CODE_PARAM2}" | grep "code=" | cut -d'=' -f2)
if ! [ -z "${AUTH_CODE1}" ]; then
    AUTH_CODE="${AUTH_CODE1}"
    echo "# Using AUTH_CODE from /tmp/auth_get_listener.log: '${AUTH_CODE}'"
fi
if ! [ -z "${AUTH_CODE2}" ]; then
    AUTH_CODE="${AUTH_CODE2}"
    echo "# Using AUTH_CODE from /tmp/auth_get_listener.log: '${AUTH_CODE}'"
fi

echo "################################################## exchange_auth_code"
check_env_var "AUTH_CODE"
# Exchange authorization code for tokens
ACCESS_TOKEN=$(exchange_auth_code ${CLIENT_ID} ${CLIENT_SECRET} ${AUTH_CODE} ${REDIRECT_URI})
echo 
cat /tmp/token_request.log
echo

echo "################################################## request UserInfo"
# lastly, request UserInfo
#ACCESS_TOKEN="%2F0ATx3LY60ftPnwh_O7BPdfu_T1ZGKKHWEgjYPuuRlu6rlswIDj43tZ3SNRvkVP3TkNEaOuQ"
curl --verbose -X GET "https://www.googleapis.com/oauth2/v1/userinfo?alt=json" -H"Authorization: Bearer ${ACCESS_TOKEN}"
