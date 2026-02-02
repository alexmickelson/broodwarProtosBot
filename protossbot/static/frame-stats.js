// frame-stats.js
let refreshInterval;

async function fetchFrameStats() {
  try {
    const response = await fetch("/api/frame-stats");
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    const data = await response.json();
    updateFrameStatsDisplay(data);
  } catch (error) {
    console.error("Error fetching frame stats:", error);
    document.getElementById("frameStatsContent").innerHTML =
      '<div style="color: var(--color-error); padding: 10px;">Error loading frame stats</div>';
  }
}

function updateFrameStatsDisplay(data) {
  const container = document.getElementById("frameStatsContent");
  container.textContent = `${data.avg_frame_time_ms.toFixed(3)} ms frame time`;
}

// Start fetching immediately
fetchFrameStats();

// Set up auto-refresh every 5 seconds
refreshInterval = setInterval(fetchFrameStats, 5000);
