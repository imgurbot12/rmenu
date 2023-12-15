/// Javasript for index.html

/* Variables */

const input = document.getElementById("search");
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
function keydown(e) {
  (e.key == "ArrowUp" || e.key == "ArrowDown") && e.preventDefault();
  _send("keydown", { "key": e.key, "ctrl": e.ctrlKey, "shift": e.shiftKey });
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

// remove active class from all current objects
function reset() {
  const classes = ["active", "selected"];
  for (const cname of classes) {
    const selected = document.getElementsByClassName(cname);
    const elems = Array.from(selected);
    elems.forEach((e) => e.classList.remove(cname));
  }
}

/// set selected-result position
function setpos(pos, smooth = false) {
  reset();
  // add selected to current position
  const current = document.getElementById(`result-${pos}`);
  if (!current) return;
  current.classList.add("selected");
  // ensure selected always within view
  current.scrollIntoView({
    behavior: smooth ? "smooth" : "auto",
    block: "center",
    inline: "center",
  });
}

// set selected-result subposition
function subpos(pos, subpos) {
  reset();
  // activate submenu
  const actions = document.getElementById(`result-${pos}-actions`);
  if (!actions) return;
  actions.classList.add("active");
  // select current subposition
  const action = document.getElementById(`result-${pos}-action-${subpos}`);
  if (!action) return;
  action.classList.add("selected");
}

/// Update Results HTML
function update(html) {
  results.innerHTML = html;
  setpos(0);
}

// Append Results HTML
function append(pos, html, smooth = false) {
  results.innerHTML += html;
  if (pos != null && pos != undefined) {
    setpos(pos, smooth);
  }
}

/* Init */

// start position at zero
setpos(0);

// capture relevant events
results.onscroll = scroll;
document.onkeydown = keydown;
document.addEventListener("DOMContentLoaded", focus);
