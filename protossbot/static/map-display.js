import {
  TILE_SIZE,
  setSvgElement,
  setUnitsGroup,
  setBaseLocationsGroup,
  setContentGroup,
  setViewBox,
  setIsPanning,
  setStartPoint,
  viewBox,
  isPanning,
  startPoint,
  svgElement,
} from "./constants.js";
import { renderMapTiles } from "./render-tiles.js";
import { fetchUnits } from "./render-units.js";
import { fetchBaseLocations } from "./render-base-locations.js";

const mapContainer = document.getElementById("mapContainer");
const mapLoading = document.getElementById("mapLoading");
const refreshIntervalSlider = document.getElementById("refreshInterval");
const intervalValueDisplay = document.getElementById("intervalValue");

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
      // Start fetching units and base locations after map is loaded
      fetchUnits();
      fetchBaseLocations();
      startRefreshInterval();
    } else {
      mapLoading.textContent = "Map data not available";
      mapLoading.classList.add("error");
    }
  } catch (error) {
    console.error("Failed to fetch map info:", error);
    mapLoading.textContent = "Error loading map data";
    mapLoading.classList.add("error");
  }
}

function startRefreshInterval() {
  // Clear existing interval if any
  if (refreshIntervalId) {
    clearInterval(refreshIntervalId);
  }
  // Start new interval
  refreshIntervalId = setInterval(() => {
    fetchUnits();
    fetchBaseLocations();
  }, refreshInterval);
}

function updateRefreshInterval(newInterval) {
  refreshInterval = newInterval;
  localStorage.setItem("mapRefreshInterval", newInterval.toString());
  startRefreshInterval();
}

function renderMap(mapData) {
  const width = mapData.map_width * TILE_SIZE;
  const height = mapData.map_height * TILE_SIZE;

  // Create SVG element
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svg.setAttribute("width", "100%");
  svg.setAttribute("height", "600px");
  svg.classList.add("map-svg");
  setSvgElement(svg);

  // Initialize viewBox
  setViewBox({ x: 0, y: 0, width: width, height: height });
  updateViewBox();

  // Create a group to hold all content (for transformations)
  const content = document.createElementNS("http://www.w3.org/2000/svg", "g");
  svg.appendChild(content);
  setContentGroup(content);

  // Render tiles
  renderMapTiles(mapData, content);

  // Create a group for units (rendered on top)
  const units = document.createElementNS("http://www.w3.org/2000/svg", "g");
  units.setAttribute("id", "units");
  content.appendChild(units);
  setUnitsGroup(units);

  // Create a group for base locations
  const bases = document.createElementNS("http://www.w3.org/2000/svg", "g");
  bases.setAttribute("id", "baseLocations");
  content.appendChild(bases);
  setBaseLocationsGroup(bases);

  // Replace loading message with the SVG
  mapContainer.innerHTML = "";
  mapContainer.appendChild(svg);

  // Add interaction handlers
  setupInteractions();
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
    setViewBox({
      x: svgX - (mouseX / rect.width) * newWidth,
      y: svgY - (mouseY / rect.height) * newHeight,
      width: newWidth,
      height: newHeight,
    });

    updateViewBox();
  });

  // Pan with mouse drag
  svgElement.addEventListener("mousedown", (e) => {
    setIsPanning(true);
    svgElement.classList.add("panning");

    const rect = svgElement.getBoundingClientRect();
    setStartPoint({
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
    });
  });

  svgElement.addEventListener("mousemove", (e) => {
    if (!isPanning) return;

    const rect = svgElement.getBoundingClientRect();
    const currentX = e.clientX - rect.left;
    const currentY = e.clientY - rect.top;

    const dx = (startPoint.x - currentX) * (viewBox.width / rect.width);
    const dy = (startPoint.y - currentY) * (viewBox.height / rect.height);

    setViewBox({
      x: viewBox.x + dx,
      y: viewBox.y + dy,
      width: viewBox.width,
      height: viewBox.height,
    });

    setStartPoint({ x: currentX, y: currentY });

    updateViewBox();
  });

  svgElement.addEventListener("mouseup", () => {
    setIsPanning(false);
    svgElement.classList.remove("panning");
  });

  svgElement.addEventListener("mouseleave", () => {
    setIsPanning(false);
    svgElement.classList.remove("panning");
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
