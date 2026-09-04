"use client";

import { Keyboard, KeyboardRefs } from "@/components/Keyboard";
import { Environment, PerspectiveCamera } from "@react-three/drei";
import gsap from "gsap";
import { useEffect, useRef } from "react";
import * as THREE from "three";

const KEYBOARD_COLUMNS = [
  ["esc", "grave", "tab", "caps", "lshift", "lcontrol"],
  ["f1", "one", "q", "a", "z", "lalt"],
  ["f2", "two", "w", "s", "x", "lwin"],
  ["f3", "three", "e", "d", "c"],
  ["f4", "four", "r", "f", "v"],
  ["f5", "five", "t", "g", "b", "space"],
  ["f6", "six", "y", "h", "n"],
  ["f7", "seven", "u", "j", "m"],
  ["f8", "eight", "i", "k", "comma"],
  ["f9", "nine", "o", "l", "period"],
  ["f10", "zero", "dash", "p", "semicolon", "slash", "ralt"],
  [
    "f11",
    "lsquarebracket",
    "quote",
    "rshift",
    "fn",
    "arrowleft",
    "rsquarebracket",
    "enter",
    "f12",
    "equal",
    "arrowup",
  ],
  [],
  [
    "del",
    "backspace",
    "backslash",
    "pagedown",
    "end",
    "arrowdown",
    "pageup",
    "arrowright",
  ],
  [],
];

const KEYCAP_AMPLITUDE = 0.035;
const HILL_SIGMA = 1.6;
const COLUMN_WIDTH = 0.03;
const SMOOTHING = 0.08;

export function Scene() {
  const keyboardRef = useRef<KeyboardRefs>(null);

  const scalingFactor =
    typeof window !== "undefined" && window.innerWidth <= 500 ? 0.5 : 1;

  useEffect(() => {
    const keyboard = keyboardRef.current;
    if (!keyboard) return;

    const prefersReducedMotion = window.matchMedia(
      "(prefers-reduced-motion: reduce)",
    ).matches;

    type KeycapItem = {
      mesh: THREE.Mesh;
      baseY: number;
      x: number;
    };

    const center = new THREE.Vector3();
    const keycapItems: KeycapItem[] = [];

    KEYBOARD_COLUMNS.flat().forEach((keyName) => {
      const keycap = keyboard.keys[keyName]?.current;
      if (keycap) {
        keycap.geometry.computeBoundingBox();
        keycap.geometry.boundingBox!.getCenter(center);
        keycapItems.push({
          mesh: keycap,
          baseY: keycap.position.y,
          x: keycap.position.x + center.x,
        });
      }
    });

    if (prefersReducedMotion) return;

    let minX = Infinity;
    let maxX = -Infinity;
    keycapItems.forEach((item) => {
      if (item.x < minX) minX = item.x;
      if (item.x > maxX) maxX = item.x;
    });

    const liftSigma = HILL_SIGMA * COLUMN_WIDTH;
    let targetX = (minX + maxX) / 2;
    let centerX = targetX;

    const onPointerMove = (event: PointerEvent) => {
      targetX =
        minX + (event.clientX / window.innerWidth) * (maxX - minX);
    };

    const tick = () => {
      centerX += (targetX - centerX) * SMOOTHING;

      const liftAt = (x: number) =>
        KEYCAP_AMPLITUDE *
        Math.exp(
          -((x - centerX) * (x - centerX)) / (2 * liftSigma * liftSigma),
        );

      keycapItems.forEach((item) => {
        item.mesh.position.y = item.baseY + liftAt(item.x);
      });
    };

    window.addEventListener("pointermove", onPointerMove);
    gsap.ticker.add(tick);

    return () => {
      window.removeEventListener("pointermove", onPointerMove);
      gsap.ticker.remove(tick);
    };
  }, []);

  return (
    <group>
      <PerspectiveCamera makeDefault position={[0, 0, 4]} fov={50} />

      <group scale={scalingFactor}>
        <group position={[0, 0, 2.2]} rotation={[Math.PI * -2 + 1.2, 0, 0]}>
          <Keyboard scale={7} ref={keyboardRef} />
        </group>
      </group>

      <Environment files={["/hdr/blue-studio.hdr"]} environmentIntensity={0.2} />

      <spotLight
        position={[-2, 1.5, 3]}
        intensity={30}
        castShadow
        penumbra={0}
        shadow-bias={-0.0002}
        shadow-normalBias={0.002}
        shadow-mapSize={1024}
      />
    </group>
  );
}
