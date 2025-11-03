import struct

# Parse the camera viewproj matrix from logs
viewproj = [
    [-0.9742786, 0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0, 0.0],  # Unknown rows
    [0.0, 0.0, 0.0, 0.0],
    [0.0, 0.0, -0.1001001, 0.0]
]

print("ViewProj matrix:")
for row in viewproj:
    print(f"  {row}")

# Camera at (0, 0, 5), looking at (0, 0, 0)
# Expected view matrix should translate by (0, 0, -5)
# Expected projection should be perspective

print("\nCamera should be at (0, 0, 5) looking at origin")
print("Near: 0.1, Far: 100.0, FOV: 60 degrees")
print("Aspect: 1.7777778 (16:9)")

# Test a vertex at the front of the cube
vertex = [-0.5, -0.5, 0.5, 1.0]  # Front bottom-left corner
print(f"\nTest vertex (front bottom-left): {vertex[:3]}")

# The cube is at origin, so in view space it should be at (0, 0, -5) + offset
# In clip space after projection...
