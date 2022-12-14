## Besedka

Besedka is a free (as in beer), open source, self-hosted commenting system for your website. It is
distributed as a single executable binary, which you can download and run on your own server.

### Features

* Portable (single binary), small (< 5kb), fast
* Free, open, no ads, no tracking, no bullshit
* Markdown
* Moderation
* No social logins, No user registration, confirmation emails, etc.
* Integrate your existing users or allow anonymous commenting
* Multiple sites
* Use your own CSS or pick one of the themes

### ⚡️ Quickstart

Download and extract the latest build from the releases page on GitHub for your architecture and
make sure the binary is executable. Next, add your site (use `besedka sites add -h` to see all
available options):

    $ besedka sites add my.blog.com --private false --anonymous true

Run the server:

    $ besedka server

On your website(s) include the following script tag:

```html
<script type="text/javascript" src="https://your-besedka-server.com/comments.js" defer></script>
```

where `https://your-besedka-server.com` is the domain pointing to the server which runs Besedka.
Finally put this div where you want your comments to appear:

```html
<div id="besedka"></div>
```

That's it!

### 🎨 Styling

To keep things light and simple there is no CSS loaded for the commenting widget by default. You can
add custom styles in your own site-wide stylesheet. Simply look at one of the existing themes for a
reference to the used classes. Alternatively, you can include a theme stylesheet:

```html
<link href="https://your-besedka-server.com/themes/default.css" rel="stylesheet" />
```

More theme contributions are welcome!

### Multiple site configuration

Besedka requires that you have explicitly added a `site` config, otherwise it will not load any
comments or allow posting of comments. To manage your sites, use the `sites` command:

    $ besedka sites --help

Please keep the `secret`s for your sites private and don't display them anywhere. You will be using
those to sign the user object when embedding Besedka on your website. If you accidentally leak a
secret to your front-end, then anyone can send whatever user info they want and cause mayhem! If
this happens, you need to immediately delete the affected config:

    $ besedka sites remove blog.mysite.com

This will only remove the site config. Comments will remain intact

### Adding moderators

A moderator can either be set with a signed user object, or one can be added from the CLI:

    $ besedka moderators add "Brian Kernighan" l3g3nd4ry_h4x0r

### Overriding the site config and the page for which comments are loaded

By default the comment widget will request the config associated with the current hostname. You can
explicitly require a different site and page by providing a config `<script>` tag:

```html
<script type="application/json" id="besedka-config">
{
  "site": "my-other-blog.example.com",
  "path": "/canonical/path/to/this/page.html"
}
</script>
```

* `site` - This is useful when you want to load a configuration for a different site or to share the
  same comments between different domains.
* `path` - Overwrite the page to which comments are linked. Set this to the canonical path in case
  your page contains dynamic parts in the URL, e.g. `/blog/page/2`

Note that these configuration values can be set by anyone.

### Signing in your existing users

If you have disabled anonymous commenting in your configuration, you will need a way to allow your
users to comment. This is done by providing a user object in a `script` tag similar to the
configuration one:

```html
<script type="application/json" id="besedka-user" data-signature="ZM5uFayLvBydsRmnlxcvhaLKCHmUy7rkQH4JavmN0bY=">
{
  "name": "John Doe",
  "moderator": true,
  "op": true,
  "avatar": "data:image/png;base64,..."
}
</script>
```

To ensure the configuration isn't tampered with by a user on the front-end, you must include a
`data-signature` attribute which contains the signed user object with the unique `secret` for the
site. Without this signature, or with an invalid one, the user info will be ignored and an error
message will be displayed instead of the commenting widget.

**IMPORTANT**: You have to keep this `secret` private on your server. Don't show it anywhere in your
pages, otherwise anyone can sign a user object, make themselves a `moderator`, and wipe all your
comments. Signing the user object **MUST** happen on your back-end.

OK, now that you have been warned, let's sign the message. First, grab the secret from the config:

    $ besedka sites get my.blog.com

You should see the `base64` encoded secret. To get the signature of a user object, you have to
obtain a SHA256 HMAC digest and then `base64` encode it. Here's how you would do that in Ruby:

```ruby
require "openssl"
require "base64"
require "json"

user = {
  name: "Jon Doe",
  moderator: true
}

secret = "BGjrlspsXqte4PMXy87wNE942gLh3pT1f+J55SE2f6U="
secret_bytes = Base64.strict_decode64(secret)
digest = OpenSSL::Digest.new('sha256')
signature = Base64.strict_encode64(OpenSSL::HMAC.digest(digest, secret_bytes, user.to_json))

puts signature
```

Note that Besedka compares the signature against the raw text value inside the
`<script type="application/json" id="besedka-user"></script>`, so whatever appears in the
tag, please ensure you sign the exact same value on your backend. Signing this:

```json
{
  "name": "Dennis Ritchie",
  "moderator": true
}
```

results in a different signature than this:

```json
{"name":"Dennis Ritchie","moderator":true}
```

All user keys are optional except when the site configuration doesn't allow anonymous posting. In
such case, you must pass a signed user with at least a `name` key that is not empty

