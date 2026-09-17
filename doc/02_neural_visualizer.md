# 🌐 02. Real-Time Neural Network Visualizer

Atena Studio includes an interactive memory network visualization interface based on particle physics simulation rendered in HTML5 Canvas at 60 FPS.

---

## 1. How to Access

1. In the application's left sidebar, click on the **"Memory"** tab (brain icon).
2. The graph instantly loads all entities and connections saved in `~/.atena/memory.atena` (or active database).

---

## 2. Graph Visual Elements

### 2.1 Node Colors and Categories

| Color | Node Type | Description | Example |
| :--- | :--- | :--- | :--- |
| 🔵 **Indigo / Blue** | `Object` | Core entities, people, or physical items | `Maria`, `Blue Pencil`, `Passport` |
| 🟢 **Teal / Green** | `Container` | Locations, drawers, folders, or boxes | `Drawer 1`, `Office`, `Box 2` |
| 🟣 **Purple / Pink** | `Attribute` | Characteristics, colors, ages, and properties | `28 years old`, `Brown eyes`, `Black color` |
| 🟠 **Amber / Orange** | `Action` | Verbs or procedural actions | `Buy`, `Store`, `Schedule` |

### 2.2 Synapses and Connections
* **Line Thickness**: Thicker lines between two nodes denote greater association strength (`weight`) and higher repetition count along that path.
* **Central Badge**: Displays the semantic relation (`STORED_WITH`, `LOCATED_IN`, `HAS_PROPERTY`).
* **Glow Effect**: Nodes and connections that are selected or active during queries receive a bright glowing aura.

---

## 3. Interactive Controls

* **Drag & Move**: Left-click and drag any node to freely reposition it.
* **Pan**: Click on the canvas background and drag to pan across the visualization.
* **Zoom**: Use mouse wheel scroll or trackpad pinch gestures to zoom in and out ($0.3\times$ to $3.0\times$).
* **Recenter**: Button in the top-left corner to reset camera position.
* **Live Search**: Type any name into the search bar to highlight matching nodes.

---

## 4. Panel Features

### 4.1 Spreading Activation Simulator
Simulates the exact memory context provided to the LLM during queries:
1. In the bottom panel, enter an anchor node (e.g., `Maria` or `Blue Pencil`);
2. Click **"Fire"**;
3. The visualizer highlights the traversed neural pathway and displays the synthesized context block:
   ```text
   [ACTIVE MEMORY: Maria -> HAS_PROPERTY:28 years old | Maria -> HAS_PROPERTY:Brown eyes]
   ```

### 4.2 Manual Fact Teaching
* Allows entering any natural language sentence (e.g., *"I live in Seattle and stored the card in my jacket pocket"*). The engine extracts nodes and edges immediately.

### 4.3 Node Inspection and Editing
* Click any node to open the side inspector showing its ID, type, complete synapse list, individual link weights, and options to reinforce or delete the entity.
