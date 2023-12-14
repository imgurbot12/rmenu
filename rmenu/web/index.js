/// Javasript for index.html

/* Variables */

const results = document.getElementById("results");

/* Functions */

/// send message back to rust
function _send(type, msg) {
  const message = JSON.stringify({ type, ...msg });
  window.webkit.messageHandlers.external.postMessage(message);
}

/// focus on search element always
function focus() {
  const search = document.getElementById("search");
  search.focus();
}

/// send search event back to rust
function search(value) {
  _send("search", { "value": value });
}

/// send keydown event back to rust
function keydown({ key, ctrlKey, shiftKey }) {
  _send("keydown", { key, "ctrl": ctrlKey, "shift": shiftKey });
}

/// send click event back to rust
function sclick(id) {
  _send("click", { click_type: "single", id });
}

/// send double-click event back to rust
function dclick(id) {
  _send("click", { click_type: "double", id });
}

/// send scroll event back to rust
function scroll() {
  const height = results.scrollHeight - results.clientHeight;
  _send("scroll", { "y": results.scrollTop, "maxy": height });
}

/// set selected-result position
function setpos(pos, smooth = false) {
  // remove selected class from all current objects
  const selected = document.getElementsByClassName("selected");
  const elems = Array.from(selected);
  elems.forEach((e) => e.classList.remove("selected"));
  // add selected to current position
  let current = document.getElementById(`result-${pos}`);
  if (!current) {
    return;
  }
  current.classList.add("selected");
  // ensure selected always within view
  current.scrollIntoView({
    behavior: smooth ? "smooth" : "auto",
    block: "center",
    inline: "center",
  });
}

/// Update Results HTML
function update(html) {
  results.innerHTML = html;
  setpos(0);
}

/* Init */

// start position at zero
setpos(0);

// capture relevant events
results.onscroll = scroll;
document.onkeydown = keydown;
document.addEventListener("DOMContentLoaded", focus);
