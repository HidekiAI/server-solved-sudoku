# Google OAUTH2 relay/proxy service

Suppose you want to integrate user-based system so that when the player quits playing sudoku in the middle of the game and continue on from where they've left off, one needs to actually associate a session ID or token for that user to pull the last known state.

Note that if it is a single-player game, the auth is really unnecessary, all we need to assume is that every connection is from the same user, even if the IP address changed since the last access.

But if it has to deal with more than single player, we do need to some how associate/map user to persisted-states.  We'll be using Google OAUTH2 service as our authenticator.  Nice thing about this service is that until the authorization token expires, that session should be valid in terms of trusting that the player says who she is, is who she really is.

In the old days, all we had to do was store the username and their session ID in a database accounting table, and be done.  Then, it became necessary that bad actors would use the same username, and the server could not trust users anymore, so we added new column called passwd, and then, if the string matched, we said she's got the right password, she's who she says she is.  Then bad actors started guessing these password, or the passwords were in plain text so it got stolen... and...  well, you get the point...  These days, I really think it's wrong for small game companies, indies, etc. to authenticate players on their own.  You let the big companies be the authoritive figure on who's who, and then you let them do the authentication.  I've seen Blizzard try this, and they've even got 2FA, etc...  But they do not realize that average players will lose their phone and other mobile devices and/or their dog chewed up the print out of the recovery keys, and so on.  It's just too much hassle!  At least for me, I no longer have an account with Blizzard because they really suck!  (I've also been banned from WoW anyways)

I like Google OAUTH2 because it's free, it's open source, it's easy to use, and it's very secure.  It's also very easy to integrate with other services, like Firebase, etc.  I'm sure Azure and AWS also has something similar, but I'm only experienced with Google, and everybody has a GMAIL account (OK, my mother only has a hotmail account, so she's not going to be able to play sudoku, but then again, she only has an iPhone and iPad, and I will not make this sudoku client available on IOS anyways, maybe on Android if it's still free to publish freewares on Google Store).

In any case, the process is simple:

1. The micro-service will be a Docker container, in which will listen to port 8080 for new connections.
2. Client connects to port 8080 of the Docker host, which will relay (port-foward) to this micro-service.
3. The micro-service is setup to relay the request to Google OAuth2 service, in which will query the player for gmail address and password, and then ask the user if it is alright to allow authorization against this sudoku service (actually, it's against just THIS micro-service, but the idea is, it's the gateway and once your knocking is heard and the door opens, the player has access to the whole cluster).
4. Google with then send tokens and expiry time in which the micro-service will store into a persisted storage, in case of reconnection.  At the time of this writing, I've prototyped for SQLite3, Redis, and PostgresSQL, but I'll be using SQLite3.
5. The client will then be notified that it's now connected to the sudoku cluster, and the client will be able to send/receive messages to/from the sudoku cluster.  But at the same time, it will have to send heartbeat pulses (keepalives) every X seconds to port 8081 (this micro-service).
6. If the client is disconnected and/or do not hear heartbeats on port 8081 for a while, the player is now disallowed access to the whole cluster.
7. If on the other hand, client is discliplined to keepalive, and the token expires, if the OAuth2 service provided a Refresh Token, it will request the OAuth2 service on behalf of the player, so that player do not require to be re-authenticated.

So why PostgresSQL?  That's because in a large-scale cluster, this micro-service is probably the most chatty, and when clients disconnects and reconnects back, the load-balancer and/or routers will not garauntee the client to connect back to the same endpoint.
