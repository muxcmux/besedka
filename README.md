## Besedka

Besedka is a free (as in beer), open source, self-hosted commenting system for your website. It is
distributed as a single executable binary, which you can download and run on your own server.

### Features

* Portable, fast, small, and easy to setup
* Free, open, no ads, no tracking, no bullshit
* Markdown
* Moderation
* No social logins, No user registration, confirmation, etc.
* Integrate your existing users or allow anonymous commenting
* [Web push](https://developer.mozilla.org/en-US/docs/Web/API/Push_API) notifications
* Honeypot for trapping bots
* Multiple configurations per domain or page
* Customisable - change the theme or use your own CSS

### ⚡️ Quickstart

Download and extract the latest build from the releases page on GitHub for your architecture. Make
the binary executable and run the server with:

    $ ./besedka server

On your website(s) you can embed comments with the following snippet:

```html
<script type="text/javascript" src="https://your-server-which-hosts-besedka.com/embed.js" defer></script>
<div id="besedka"></div>
```

That's it!

### Configuration

Besedka can be configured from the command line or per instance with a configuration object. A
default config will be created upon running the program for the first time. To see what it is run:

    $ ./besedka config

You can have different settings per hostname:

    $ ./besedka config set --site blog.example.com --theme sleek --moderated true

Please keep the `secret`s for your sites private and don't display them anywhere. You will be using
those to sign the configuration object when embedding Besedka on your website. If you accidentally
leak a secret to your front-end, then anyone can send whatever configuration they want and cause
mayhem! If this happens, you need to immediately reset the secret for the affected config using:

    $ ./besedka config reset-secret

See other available config commands:

    $ ./besedka config --help

### Adding moderators

A moderator can either be set with a signed configuration object, or one can be added from the CLI:

    $ ./besedka moderators add --name "Brian Kernighan" --pass l3g3nd4ry_h4x0r

Note that logging in from the comment widget UI is only available for moderators created from the
CLI. Moderators linked via a config object will not be able to login via the widget UI as Besedka
doesn't store any passwords for your users.

### Configuration per page

You can configure each embedded instance of Besedka by providing a config object inside a `<script
type="application/json" id="besedka-config>"` tag:

```html
<script type="application/json" id="besedka-config" data-signature="ZM5uFayLvBydsRmnlxcvhaLKCHmUy7rkQH4JavmN0bY=">
{
  "site": "my-other-blog.example.com",
  "page": "/canonical/path/to/this/page.html",
  "anonymous_comments": true,
  "moderated": true,
  "comments_per_page": 10,
  "replies_per_comment": 50,
  "minutes_to_edit": 0,
  "theme": "modern",
  "user": {
    "id": 42,
    "username": "john@example.com",
    "name": "John Doe",
    "moderator": true,
    "avatar": "data:image/png;base64,..."
  }
}
</script>
```

The configuration object takes the same keys as the command line plus a few additional ones, most
notably the `user` object.

* `site` - Overwrite the site comments on this page will be associated with. This is useful when you
  want to share the same comments between different domains.
* `path` - Overwrite the page the comments are linked to. Set this to the canonical path in case
  your page contains dynamic parts in the URL, e.g. `/blog/page/2`

If set, the following keys will take precedence over the values configured for the current site and
the default config:

* `anonymous_comments`
* `moderated`
* `comments_per_page`
* `replies_per_comment`
* `minutes_to_edit`
* `theme`

The `user` object is used to link your existing users to comments. They this works is by specifying
your currently logged in user's `id` in the `user` object. You can optionally pass `username`,
`name`, `moderator`, and `avatar` keys.

#### Signing the config

To ensure the configuration isn't tampered with by a user on the front-end, you must include a
`data-signature` attribute which contains the signed config object with the unique `secret` for the
site. Without this signature, or with an invalid one, the settings will be ignored and an error
message will be displayed instead of the commenting widget.

**IMPORTANT**: You have to keep this `secret` private on your server. Don't show it anywhere in your
pages, otherwise anyone can sign a configuration object, make themselves a `moderator`, and wipe all
your comments. Signing the config object **MUST** happen on your back-end.

Ok, now that you have been warned, to get the signature of a config object, you have to use a SHA256
HMAC and `base64` encode it. Here's how you would do that in Ruby:

```ruby
config = {
  user: {
    id: 42,
    moderator: true
  }
}
base64_secret = "your-besedka-secret"
digest = OpenSSL::Digest.new('sha256')
signature = Base64.strict_encode64(OpenSSL::HMAC.digest(digest, base64_secret, config.to_json))
```

Note that Besedka compares the signature against a "non-pretty" `json` object. Signing this:

```json
{
  "user": {
    "id": 42,
    "moderator": true
  }
}
```

results in a different signature than this:

```json
{"user":{"id":42,"moderator":true}}
```

When signing, please sign the minified version without new lines and unnecessary spaces.

### Running Besedka as a service

You'd most probably want to run besedka all the time and ensure it restarts when your server
restarts or if (when) it crashes. Please have a look at the `etc` dir for a sample `systemd` script
