const presetButtons = document.querySelectorAll(".preset-buttons button");
const statusDiv = document.getElementById("status");

function highlightButton(speed) {
  presetButtons.forEach((btn) => {
    if (Number(btn.dataset.speed) === speed) {
      btn.classList.add("active");
    } else {
      btn.classList.remove("active");
    }
  });
}

async function fetchCurrentSpeed() {
  const response = await fetch("http://127.0.0.1:3333/api/speed");
  if (response.ok) {
    const data = await response.json();
    highlightButton(data.speed);
    // Clear any previous error message
    statusDiv.textContent = "";
    statusDiv.className = "status";
  } else {
    showStatus("Unable to connect to bot", "error");
  }
}

export async function setSpeed(speed) {
  const response = await fetch("http://127.0.0.1:3333/api/speed", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ speed: speed }),
  });

  if (response.ok) {
    const data = await response.json();
    highlightButton(data.speed);
    // Clear any previous status message on success
    statusDiv.textContent = "";
    statusDiv.className = "status";
  } else {
    showStatus("Failed to update speed", "error");
  }
}

function showStatus(message, type) {
  statusDiv.textContent = message;
  statusDiv.className = `status ${type} visible`;
  setTimeout(() => {
    statusDiv.classList.remove("visible");
  }, 3000);
}

fetchCurrentSpeed();
setInterval(fetchCurrentSpeed, 5000);

// Make setSpeed available globally for onclick handlers
window.setSpeed = setSpeed;
