export type ButtonZone = { button: string; label: string; path: string };
export type MouseDrawing = {
  image: string;
  width: number;
  height: number;
  zones: ButtonZone[];
};

// Coordinates are traced in ORIGINAL image pixels, never percentages of the
// surrounding card. Image and hit paths share the same SVG viewBox/transform.
// Leave the physical seams, wheel surround, LED and chassis unpainted.
export const mouseDrawings: Record<"x1" | "x1-side", MouseDrawing> = {
  x1: {
    image: "/assets/x1/attack-shark-x1-top.png", width: 1229, height: 1280,
    zones: [
      { button: "Button 1", label: "Left click", path: "M355 138 L571 44 L572 340 Q572 370 612 375 L616 375 L616 568 C544 568 464 570 404 573 Q367 574 363 532 C355 421 349 236 355 138 Z" },
      { button: "Button 2", label: "Right click", path: "M666 44 L879 139 C883 243 883 413 874 534 Q871 573 835 573 C761 570 690 570 623 569 L623 375 L635 374 Q666 370 666 341 Z" },
      { button: "Button 3", label: "Wheel click", path: "M596 195 Q619 189 642 196 Q646 197 646 205 L646 347 Q645 357 638 358 L600 358 Q592 357 592 348 L592 204 Q592 197 596 195 Z" },
      { button: "Button 4", label: "Side 1", path: "M339 426 Q343 422 346 430 L352 566 L341 575 Q337 572 337 563 L336 443 Z" },
      { button: "Button 5", label: "Side 2", path: "M343 598 Q350 593 353 599 L355 745 Q355 756 351 753 Q339 746 339 732 L339 613 Z" },
    ],
  },
  "x1-side": {
    image: "/assets/x1/attack-shark-x1-side.png", width: 1200, height: 400,
    zones: [
      { button: "Button 4", label: "Side 1", path: "M399 194 C443 184 490 176 545 172 Q549 172 549 177 L549 229 C515 230 476 235 444 242 Q425 245 408 229 Q392 215 391 204 Q390 198 399 194 Z" },
      { button: "Button 5", label: "Side 2", path: "M578 172 C627 172 681 179 724 190 Q737 193 736 200 Q733 211 706 232 Q694 242 679 239 C644 233 609 232 578 232 Q574 231 574 226 L574 179 Q574 172 578 172 Z" },
    ],
  },
};
