// Shared constants
export const TILE_SIZE = 4; // Display size for each tile position
export const PIXEL_TO_DISPLAY_SCALE = 0.125; // 1 pixel = 0.125 display pixels (32 pixels = 4 display pixels)

// Global SVG groups - will be set by renderMap
export let svgElement = null;
export let unitsGroup = null;
export let baseLocationsGroup = null;
export let contentGroup = null;

// Pan and zoom state
export let viewBox = { x: 0, y: 0, width: 0, height: 0 };
export let isPanning = false;
export let startPoint = { x: 0, y: 0 };
export let scale = 1;

// Setter functions to update module-level variables
export function setSvgElement(value) {
  svgElement = value;
}
export function setUnitsGroup(value) {
  unitsGroup = value;
}
export function setBaseLocationsGroup(value) {
  baseLocationsGroup = value;
}
export function setContentGroup(value) {
  contentGroup = value;
}
export function setViewBox(value) {
  viewBox = value;
}
export function setIsPanning(value) {
  isPanning = value;
}
export function setStartPoint(value) {
  startPoint = value;
}
