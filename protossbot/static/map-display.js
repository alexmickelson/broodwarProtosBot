const mapContainer = document.getElementById("mapContainer");
const mapLoading = document.getElementById("mapLoading");
const refreshIntervalSlider = document.getElementById("refreshInterval");
const intervalValueDisplay = document.getElementById("intervalValue");

const TILE_SIZE = 4; // Display size for each tile position
const PIXEL_TO_DISPLAY_SCALE = 0.125; // 1 pixel = 0.125 display pixels (32 pixels = 4 display pixels)

let svgElement = null;
let unitsGroup = null;
let contentGroup = null;

// Pan and zoom state
let viewBox = { x: 0, y: 0, width: 0, height: 0 };
let isPanning = false;
let startPoint = { x: 0, y: 0 };
let scale = 1;

// Refresh interval state
let refreshInterval =
  parseInt(localStorage.getItem("mapRefreshInterval")) || 1000;
let refreshIntervalId = null;

async function fetchMapInfo() {
  try {
    const response = await fetch("http://127.0.0.1:3333/api/map-info");
    if (response.ok) {
      const data = await response.json();
      renderMap(data);
      // Start fetching units after map is loaded
      fetchUnits();
      startRefreshInterval();
    } else {
      mapLoading.textContent = "Map data not available";
      mapLoading.style.color = "#f87171";
    }
  } catch (error) {
    console.error("Failed to fetch map info:", error);
    mapLoading.textContent = "Error loading map data";
    mapLoading.style.color = "#f87171";
  }
}

function startRefreshInterval() {
  // Clear existing interval if any
  if (refreshIntervalId) {
    clearInterval(refreshIntervalId);
  }
  // Start new interval
  refreshIntervalId = setInterval(fetchUnits, refreshInterval);
}

function updateRefreshInterval(newInterval) {
  refreshInterval = newInterval;
  localStorage.setItem("mapRefreshInterval", newInterval.toString());
  startRefreshInterval();
}

async function fetchUnits() {
  try {
    const response = await fetch("http://127.0.0.1:3333/api/unit-info");
    if (response.ok) {
      const data = await response.json();
      renderUnits(data.units);
    }
  } catch (error) {
    console.error("Failed to fetch unit info:", error);
  }
}

function renderMap(mapData) {
  const width = mapData.map_width * TILE_SIZE;
  const height = mapData.map_height * TILE_SIZE;

  // Create SVG element
  svgElement = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svgElement.setAttribute("width", "100%");
  svgElement.setAttribute("height", "600px");
  svgElement.style.display = "block";
  svgElement.style.border = "1px solid #444";
  svgElement.style.cursor = "grab";

  // Initialize viewBox
  viewBox = { x: 0, y: 0, width: width, height: height };
  updateViewBox();

  // Create a group to hold all content (for transformations)
  contentGroup = document.createElementNS("http://www.w3.org/2000/svg", "g");
  svgElement.appendChild(contentGroup);

  // Create a map of tiles for quick lookup
  const tileMap = new Map();
  mapData.tiles.forEach((tile) => {
    const key = `${tile.x},${tile.y}`;
    tileMap.set(key, tile);
  });

  // Render all tiles
  for (let y = 0; y < mapData.map_height; y++) {
    for (let x = 0; x < mapData.map_width; x++) {
      const key = `${x},${y}`;
      const tile = tileMap.get(key);

      const rect = document.createElementNS(
        "http://www.w3.org/2000/svg",
        "rect",
      );
      rect.setAttribute("x", x * TILE_SIZE);
      rect.setAttribute("y", y * TILE_SIZE);
      rect.setAttribute("width", TILE_SIZE);
      rect.setAttribute("height", TILE_SIZE);

      // Color based on walkability
      if (tile && tile.is_walkable) {
        rect.setAttribute("fill", "#666666"); // gray for walkable
      } else {
        rect.setAttribute("fill", "#000000"); // black for unwalkable
      }

      contentGroup.appendChild(rect);
    }
  }

  // Create a group for units (rendered on top)
  unitsGroup = document.createElementNS("http://www.w3.org/2000/svg", "g");
  unitsGroup.setAttribute("id", "units");
  contentGroup.appendChild(unitsGroup);

  // Replace loading message with the SVG
  mapContainer.innerHTML = "";
  mapContainer.appendChild(svgElement);

  // Add interaction handlers
  setupInteractions();
}

function renderUnits(units) {
  if (!unitsGroup) return;

  // Clear existing units
  unitsGroup.innerHTML = "";

  // Render each unit
  units.forEach((unit) => {
    const x = unit.x * PIXEL_TO_DISPLAY_SCALE;
    const y = unit.y * PIXEL_TO_DISPLAY_SCALE;
    const width = unit.width * PIXEL_TO_DISPLAY_SCALE;
    const height = unit.height * PIXEL_TO_DISPLAY_SCALE;

    const rect = document.createElementNS("http://www.w3.org/2000/svg", "rect");
    rect.setAttribute("x", x - width / 2);
    rect.setAttribute("y", y - height / 2);
    rect.setAttribute("width", width);
    rect.setAttribute("height", height);
    rect.setAttribute("fill", getUnitColor(unit.player_id));
    rect.setAttribute("stroke", "#ffffff");
    rect.setAttribute("stroke-width", "0.5");
    rect.setAttribute("opacity", "0.8");

    // Add tooltip
    const title = document.createElementNS(
      "http://www.w3.org/2000/svg",
      "title",
    );
    title.textContent = `ID: ${unit.unit_id}\nPlayer: ${unit.player_name || "Unknown"}`;
    rect.appendChild(title);

    unitsGroup.appendChild(rect);
  });
}

function getUnitColor(playerId) {
  // Color units based on player ID
  const colors = [
    "#ff0000", // Red (Player 0)
    "#0000ff", // Blue (Player 1)
    "#00ff00", // Green (Player 2)
    "#ffff00", // Yellow (Player 3)
    "#ff00ff", // Purple (Player 4)
    "#00ffff", // Cyan (Player 5)
    "#ff8800", // Orange (Player 6)
    "#ffffff", // White (Player 7)
  ];

  if (playerId !== null && playerId >= 0 && playerId < colors.length) {
    return colors[playerId];
  }
  return "#888888"; // Gray for unknown/neutral
}

function setupInteractions() {
  if (!svgElement) return;

  // Zoom with mouse wheel
  svgElement.addEventListener("wheel", (e) => {
    e.preventDefault();

    const rect = svgElement.getBoundingClientRect();
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    // Convert mouse position to SVG coordinates
    const svgX = viewBox.x + (mouseX / rect.width) * viewBox.width;
    const svgY = viewBox.y + (mouseY / rect.height) * viewBox.height;

    // Zoom factor
    const zoomFactor = e.deltaY < 0 ? 0.9 : 1.1;

    // Calculate new viewBox dimensions
    const newWidth = viewBox.width * zoomFactor;
    const newHeight = viewBox.height * zoomFactor;

    // Adjust position to zoom towards mouse
    viewBox.x = svgX - (mouseX / rect.width) * newWidth;
    viewBox.y = svgY - (mouseY / rect.height) * newHeight;
    viewBox.width = newWidth;
    viewBox.height = newHeight;

    updateViewBox();
  });

  // Pan with mouse drag
  svgElement.addEventListener("mousedown", (e) => {
    isPanning = true;
    svgElement.style.cursor = "grabbing";

    const rect = svgElement.getBoundingClientRect();
    startPoint = {
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
    };
  });

  svgElement.addEventListener("mousemove", (e) => {
    if (!isPanning) return;

    const rect = svgElement.getBoundingClientRect();
    const currentX = e.clientX - rect.left;
    const currentY = e.clientY - rect.top;

    const dx = (startPoint.x - currentX) * (viewBox.width / rect.width);
    const dy = (startPoint.y - currentY) * (viewBox.height / rect.height);

    viewBox.x += dx;
    viewBox.y += dy;

    startPoint = { x: currentX, y: currentY };

    updateViewBox();
  });

  svgElement.addEventListener("mouseup", () => {
    isPanning = false;
    svgElement.style.cursor = "grab";
  });

  svgElement.addEventListener("mouseleave", () => {
    isPanning = false;
    svgElement.style.cursor = "grab";
  });
}

function updateViewBox() {
  if (!svgElement) return;
  svgElement.setAttribute(
    "viewBox",
    `${viewBox.x} ${viewBox.y} ${viewBox.width} ${viewBox.height}`,
  );
}

// Initialize slider with saved value
if (refreshIntervalSlider && intervalValueDisplay) {
  refreshIntervalSlider.value = refreshInterval;
  intervalValueDisplay.textContent = refreshInterval;

  refreshIntervalSlider.addEventListener("input", (e) => {
    const value = parseInt(e.target.value);
    intervalValueDisplay.textContent = value;
    updateRefreshInterval(value);
  });
}

// Fetch map info once on page load
fetchMapInfo();
