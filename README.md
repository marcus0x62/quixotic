# What is this?

 A Markov-chain nonsense generator that will slightly alter text and HTML content from
static websites to make the content less useful to bots and LLMs.  More information and instructions
are [on the web page](https://marcusb.org/hacks/quixotic.html)

# MSRV

 You'll need 1.81.0 or later to build Quixotic.

# What's New
* March-28-2025: Bump dependencies to fix issue #4. Change rand call sites to use new API names.
* January-25-2025: Fixed a bug related to extension-less files.  Documented MSRV.
* January-17-2025: add support (enabled by default) for image
  scrambling. This works by substituting images randomly (by default,
  around 40% of the images in your input directory should be selected
  for substitution; those selected will be replaced with another image
  from your input directory at random.) You can change the threshold
  with ```--scramble-images <percent>```, i.e, set ```--scramble-images
  0.75``` to scramble 75% of your site's images.
