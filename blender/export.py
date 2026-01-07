import bpy
import os
import yaml
from mathutils import Matrix

# =========================
# CONFIG
# =========================
EXPORT_ROOT = bpy.path.abspath("//export")
MESH_DIR = os.path.join(EXPORT_ROOT, "meshes")
SCENE_FILE = os.path.join(EXPORT_ROOT, "scene.yaml")

# =========================
# HELPERS
# =========================
def vec(v):
    return [float(v.x), float(v.y), float(v.z)]

def ensure_dirs():
    os.makedirs(MESH_DIR, exist_ok=True)

def export_obj(obj, filepath):
    bpy.ops.object.select_all(action='DESELECT')
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj

    bpy.ops.export_scene.obj(
        filepath=filepath,
        use_selection=True,
        use_materials=False,
        axis_forward='-Z',
        axis_up='Y'
    )

def object_transform(obj):
    return {
        "location": vec(obj.location),
        "rotation": vec(obj.rotation_euler),
        "scale": vec(obj.scale),
    }

# =========================
# MAIN EXPORT
# =========================
def export_scene():
    ensure_dirs()

    scene_data = {
        "scene": {
            "units": "meters",
            "objects": [],
            "cameras": [],
            "lights": [],
        }
    }

    for obj in bpy.context.scene.objects:
        # -------- MESHES --------
        if obj.type == 'MESH':
            obj_path = f"meshes/{obj.name}.obj"
            export_obj(obj, os.path.join(EXPORT_ROOT, obj_path))

            scene_data["scene"]["objects"].append({
                "name": obj.name,
                "mesh": obj_path,
                "transform": object_transform(obj),
            })

            # -------- CAMERAS --------
        elif obj.type == 'CAMERA':
            cam = obj.data
            scene_data["scene"]["cameras"].append({
                "name": obj.name,
                "transform": {
                    "location": vec(obj.location),
                    "rotation": vec(obj.rotation_euler),
                },
                "lens": {
                    "type": "perspective" if cam.type == 'PERSP' else "orthographic",
                    "focal_length_mm": cam.lens,
                    "sensor_width_mm": cam.sensor_width,
                }
            })

        # -------- LIGHTS --------
        elif obj.type == 'LIGHT':
            light = obj.data
            scene_data["scene"]["lights"].append({
                "name": obj.name,
                "type": light.type,
                "energy": light.energy,
                "color": list(light.color),
                "transform": {
                    "location": vec(obj.location),
                    "rotation": vec(obj.rotation_euler),
                }
            })

    with open(SCENE_FILE, "w") as f:
        yaml.dump(scene_data, f, sort_keys=False)

    print(f"Exported scene to {SCENE_FILE}")

# =========================
# RUN
# =========================
    export_scene()