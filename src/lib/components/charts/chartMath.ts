export interface Point {
  x: number;
  y: number;
}

export function linearScale(
  val: number,
  domainMin: number,
  domainMax: number,
  rangeMin: number,
  rangeMax: number
): number {
  if (domainMax === domainMin) {
    return (rangeMin + rangeMax) / 2;
  }
  const ratio = (val - domainMin) / (domainMax - domainMin);
  return rangeMin + ratio * (rangeMax - rangeMin);
}

export function generateNiceTicks(min: number, max: number, tickCount = 5): number[] {
  if (min === max) {
    return [min];
  }
  const span = max - min;
  const rawStep = span / (tickCount - 1);
  const power = Math.floor(Math.log10(Math.max(1, Math.abs(rawStep))));
  const magnitude = 10 ** power;
  const residual = rawStep / magnitude;

  let niceStep = magnitude;
  if (residual > 5) {
    niceStep = 10 * magnitude;
  } else if (residual > 2) {
    niceStep = 5 * magnitude;
  } else if (residual > 1) {
    niceStep = 2 * magnitude;
  }

  const start = Math.floor(min / niceStep) * niceStep;
  const end = Math.ceil(max / niceStep) * niceStep;

  const ticks: number[] = [];
  for (let current = start; current <= end; current += niceStep) {
    ticks.push(current);
  }
  return ticks;
}

export function buildStraightPath(points: Point[]): string {
  if (points.length === 0) return '';
  return points.reduce((acc, pt, idx) => {
    return idx === 0
      ? `M ${pt.x.toFixed(1)} ${pt.y.toFixed(1)}`
      : `${acc} L ${pt.x.toFixed(1)} ${pt.y.toFixed(1)}`;
  }, '');
}

export function buildMonotonePath(points: Point[]): string {
  if (points.length === 0) return '';
  if (points.length === 1) return `M ${points[0].x} ${points[0].y}`;
  if (points.length === 2) {
    return `M ${points[0].x} ${points[0].y} L ${points[1].x} ${points[1].y}`;
  }

  let d = `M ${points[0].x.toFixed(1)} ${points[0].y.toFixed(1)}`;
  for (let i = 0; i < points.length - 1; i++) {
    const p0 = points[Math.max(0, i - 1)];
    const p1 = points[i];
    const p2 = points[i + 1];
    const p3 = points[Math.min(points.length - 1, i + 2)];

    const cp1x = p1.x + (p2.x - p0.x) / 6;
    const cp1y = p1.y + (p2.y - p0.y) / 6;
    const cp2x = p2.x - (p3.x - p1.x) / 6;
    const cp2y = p2.y - (p3.y - p1.y) / 6;

    d += ` C ${cp1x.toFixed(1)} ${cp1y.toFixed(1)}, ${cp2x.toFixed(1)} ${cp2y.toFixed(1)}, ${p2.x.toFixed(1)} ${p2.y.toFixed(1)}`;
  }
  return d;
}

export function buildAreaPath(points: Point[], zeroY: number): string {
  if (points.length === 0) return '';
  const linePath = buildMonotonePath(points);
  const first = points[0];
  const last = points[points.length - 1];
  return `${linePath} L ${last.x.toFixed(1)} ${zeroY.toFixed(1)} L ${first.x.toFixed(1)} ${zeroY.toFixed(1)} Z`;
}

export interface SankeyNodeInput {
  id: string;
  label: string;
  value: number;
  color: string;
  column: number;
}

export interface SankeyLinkInput {
  source: string;
  target: string;
  value: number;
  color: string;
}

export interface SankeyLayoutResult {
  nodes: (SankeyNodeInput & { y: number; height: number })[];
  links: (SankeyLinkInput & { path: string })[];
  width: number;
  actualHeight: number;
  colWidth: number;
}

export function calculateSankeyLayout(
  nodesInput: SankeyNodeInput[],
  linksInput: SankeyLinkInput[],
  width = 800,
  baseHeight = 460,
  colWidth = 180,
  padding = 12
): SankeyLayoutResult {
  const nodes = nodesInput.map((n) => ({ ...n, y: 0, height: 0 }));
  const links = linksInput.map((l) => ({ ...l, path: '' }));

  if (nodes.length === 0) {
    return { nodes: [], links: [], width, actualHeight: baseHeight, colWidth };
  }

  const cols: (typeof nodes)[] = [[], [], []];
  nodes.forEach((n) => {
    if (cols[n.column]) cols[n.column].push(n);
  });

  const colTotals = cols.map((col) => col.reduce((sum, n) => sum + n.value, 0));
  const maxTotal = Math.max(...colTotals, 1);
  const maxNodesInCol = Math.max(...cols.map((c) => c.length), 1);
  const minRequiredHeight = maxNodesInCol * 20 + (maxNodesInCol - 1) * padding;
  const actualHeight = Math.max(baseHeight, minRequiredHeight);

  let scale = (actualHeight - (maxNodesInCol - 1) * padding) / maxTotal;
  if (!isFinite(scale) || scale <= 0) scale = 1;

  cols.forEach((colNodes, colIdx) => {
    colNodes.sort((a, b) => b.value - a.value);
    let yOffset = 0;
    if (colIdx === 1 && colNodes.length === 1) {
      const h = colNodes[0].value * scale;
      yOffset = (actualHeight - h) / 2;
    } else {
      const colTotalHeight =
        colNodes.reduce((sum, n) => sum + n.value * scale, 0) + (colNodes.length - 1) * padding;
      yOffset = (actualHeight - colTotalHeight) / 2;
    }

    colNodes.forEach((node) => {
      node.height = Math.max(4, node.value * scale);
      node.y = yOffset;
      yOffset += node.height + padding;
    });
  });

  const nodeOffsets: Record<string, { left: number; right: number }> = {};
  nodes.forEach((n) => {
    nodeOffsets[n.id] = { left: n.y, right: n.y };
  });

  links.forEach((link) => {
    const sourceNode = nodes.find((n) => n.id === link.source);
    const targetNode = nodes.find((n) => n.id === link.target);
    if (!sourceNode || !targetNode) return;

    const linkHeight = Math.max(1, link.value * scale);
    const startX = sourceNode.column === 0 ? colWidth : width / 2 + colWidth / 2;
    const startY = nodeOffsets[sourceNode.id].right;
    nodeOffsets[sourceNode.id].right += linkHeight;

    const endX = targetNode.column === 2 ? width - colWidth : width / 2 - colWidth / 2;
    const endY = nodeOffsets[targetNode.id].left;
    nodeOffsets[targetNode.id].left += linkHeight;

    const curvature = 0.5;
    const xOffset = (endX - startX) * curvature;

    link.path = `M ${startX.toFixed(1)} ${startY.toFixed(1)} C ${(startX + xOffset).toFixed(1)} ${startY.toFixed(1)}, ${(endX - xOffset).toFixed(1)} ${endY.toFixed(1)}, ${endX.toFixed(1)} ${endY.toFixed(1)} L ${endX.toFixed(1)} ${(endY + linkHeight).toFixed(1)} C ${(endX - xOffset).toFixed(1)} ${(endY + linkHeight).toFixed(1)}, ${(startX + xOffset).toFixed(1)} ${(startY + linkHeight).toFixed(1)}, ${startX.toFixed(1)} ${(startY + linkHeight).toFixed(1)} Z`;
  });

  return { nodes, links, width, actualHeight, colWidth };
}
