# Rust Game Creator

A native Rust world builder and object editor, growing toward a toolkit for making
custom games, including online roleplay games.

**Working now:** colored world construction, primitive and building-part placement,
grab/move/rotate/resize controls, local object grids, shaped cuts and additions,
freehand shaping, extrusion, mirrored editing, hollowing, joining and subtraction,
saved reusable builds, undo/redo, and a first person walk preview.

**In development:** surface painting. Stroke data and persistence foundations are
present; the paint interface, drawing input and surface rendering are not connected yet.

**Planned:** tablet pressure, texture painting and layers, a full custom mission
builder, character animation tools, vehicles, jobs, businesses, housing and multiplayer
RP systems. These are roadmap features, not available in this release.

Windows is required for the current editor. Build with stable Rust using
`cargo build --release`, or double-click **Build Project Grove.bat**, then launch.
The local folder and launcher names still use the earlier Project Grove name.
No external Rust crates or GTA installation are required. The bundled starter scene
uses original procedural geometry.

## Current editor guide

Double-click **Launch Project Grove.bat**. Everything is contained in this folder.

## The simple way to build

1. Choose a wall, floor, roof, road, or other part from the toolbox.
2. Move your mouse over the world and click to stamp it. **R** turns the next part
   90 degrees; the wheel turns it in 15 degree steps. **Shift + wheel** changes its height.
3. Press **1**, click a piece, then press **G** to grab it.
4. Move the mouse to adjust it. **R** cycles **Move -> Turn -> Size** while you hold it.
   **X / Y / Z** locks an axis; press the same axis again to release it.
5. **Click or Enter** places it. **Escape** restores everything to before the grab.

The current action and useful keys stay visible above the world. **F1** opens the guide.
Scroll over the left toolbox for more controls. **Exact properties** expands numerical
position, dimensions, X/Y/Z rotation, color and name fields. Enter applies those values
only to the last selected object. It preserves any cuts in that object.

Hold **right mouse** to look, and use **WASD** to fly. **Q/E** lowers/raises the camera;
**Shift** speeds it up. You can move the camera while holding a piece. **F** frames
selection, **Home** returns to the overview, and **Top** gives a view from above.

## Grids and axes

- **[ / ]** changes world snap spacing: off, 0.1, 0.25, 0.5, 1, 2, or 4 meters.
- **B** shows/hides the construction grid independently of snapping.
- **Origin** aligns the grid to the last selected piece; clicking again resets the origin.
- **N** toggles Join, which attracts nearby object edges together in horizontal movement.
- **X/Y/Z** constrains moving, turning, or resizing. Rotation supports all three axes.
  Free turning uses vertical Z. Axis resizing changes the corresponding object dimension.
- **Shift** temporarily removes movement snapping and makes turning/resizing more precise.
- **Arrows** nudge horizontally; **Page Up/Down** nudges height. Grid sets the increment.
- The visible construction grid follows your view and becomes coarser at a distance.

## Cut and customize objects

Select an object and press **E** or **Edit shape**. It receives its own editable cell grid.
The turquoise lines show that grid on the object. Drag a rectangle on any face and release
it to cut cells out. A colored preview marks the region while dragging.

**Cut: through** removes that rectangular region through the object, making a doorway,
window, tunnel, or opening. **Cut: 1 layer** removes just the outer layer for a recess.
Switch **Cut cells** to **Fill cells** to put cells back. Fill targets the original bounding
faces, including where there is already a hole. Press **E** again to finish editing.

**Object grid** cycles 8, 16 and 32 cells across the object's longest dimension. Other axes
use proportional cell counts. Each object keeps its own resolution. Coarser grids can lose
small details; **Ctrl+Z** restores the previous grid. This is cell-based shape editing:
curves become stepped when converted to an editable grid. The grid rotates and scales with
the object. All removed faces are actual missing geometry; openings affect walk collision.

Cuts, fills, grid changes, and complete grab operations can be undone. Resizing or renaming
a cut object keeps its cuts. Deleted cells are saved in the scene and in reusable builds.
An entirely hollowed object is invisible; undo the cut or fill cells before leaving its edit
mode if you want to keep working on it.

## Houses, buildings and streets

Ready parts include **walls, floors, door sections, window sections, pitched roofs,
room shells, house shells, garage shells, roads, pavements, curbs, crossroads,
T junctions, fences, stairs, and ramps**. Multi-piece stamps remain individually editable.
They are all selected immediately after placing, so you can move or copy the whole stamp.

**Repeat line** stamps a row: click START, then END. Pieces repeat at their footprint
spacing, keeping adjacent wall/road modules touching. The last piece can extend slightly
past your endpoint so modules retain their full size. Set the piece's rotation before
placing the row. One line supports up to 256 stamps and is one undo operation.

**Ctrl+click** adds/removes objects from the selection. Palette colors, duplicate, delete,
move/turn/size, collision and lock controls operate on the selection. Locked objects are
protected from edits and deletion. The foundation starts locked.

**Ctrl+D** copies selection and grabs the copy, ready to move. The toolbox Copy button
copies without immediately grabbing. **Delete** removes selected unlocked pieces.
**Ctrl+Z / Ctrl+Y** undo/redo up to 100 scene edits.

**Save build** opens a save window so you can name each building or custom object in
`prefabs/`. **Load build** opens that library so you can choose a saved piece and stamp
it into the world. All cuts and component colors are preserved. Replacing an existing
build keeps a backup of its previous version.

## Walk and save

**Tab** switches between building and walking. In walk mode, **WASD** moves,
**right mouse** looks, **Shift** runs, and **Space** jumps. **Set player spawn** moves the
spawn to the top of the selected object or below the camera when nothing is selected.
The small mannequin is an original procedural animation reference, not an NPC or ragdoll.

**Save / Ctrl+S** writes `worlds/sandbox.grove`; the previous file becomes
`worlds/sandbox.grove.bak`. **Load** reloads that scene and can be undone.
Unsaved work is written to `worlds/recovery.grove` on normal close and restored when newer
than the main save. Save commits that recovery as the main scene. Save regularly while working.

Scene format 3 stores full rotations and object grids. Earlier scene versions still load.
**F12** captures `sandbox-preview.bmp`. All code is in `src/`; **Build Project Grove.bat**
rebuilds the executable. No game installation or external crates are required.

## Current limits

This is an original colored blockout editor and local walk preview. Textures, UV painting,
mesh vertex editing, curved road tools, and multiplayer are later work. Grid shaping is
bounded to 32 cells per axis per object; large or detailed shapes can be assembled from
several objects. Walk collision uses a simple controller, with conservative edge handling
for curved/tilted shapes. The current format supports up to 50,000 objects, but practical
performance depends on the number of generated faces.

## Object editor 0.3

Press **E** on a selected unlocked object. **V** cycles Box, Circle, Triangle,
Arch, Diamond, Ball and Draw freehand brushes. Drag across a face; release to commit.
The colored preview shows the shape. **C** switches Cut/Add; **P** selects Pull out.
**[ / ]** adjusts depth in cells. Wheel changes depth, or freehand brush width.
**X/Y/Z** toggles symmetry across the object's own axes while editing.

Through cuts the entire object depth. Turn it off to use the depth setting for
recesses and additions. Pull out extends the selected solid face by the depth setting;
it ignores Through. Expand space adds two empty cell layers around each side while
preserving existing world geometry. Add can fill this new space. Canvas growth is
limited to 64 cells per axis; resolution conversion still offers 8/16/32.

Hollow object removes the interior while retaining walls of the chosen depth.
Round edges applies one grid-based softening pass. Both are undoable. These tools
retain the cell-based appearance; they do not create a smooth editable polygon mesh.

Join pieces combines unlocked selected objects into one editable grid. It uses the
chosen grid resolution and the last piece's color, so small details or mixed colors
can be lost; undo restores the original parts. Use as cutters subtracts the selected
shapes from the **last selected object**: select cutters first, then Ctrl+click the
object you want to cut. Cutter objects are consumed on success. Undo restores all.

Custom shapes, expanded grids and cuts save in the scene and named building library.
