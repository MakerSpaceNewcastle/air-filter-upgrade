module ArrangeCentres2D(centres) {
  d = centres / 2;

  for(x = [-d[0], d[0]]) {
    for(y = [-d[1], d[1]]) {
      translate([x, y]) {
        children();
      }
    }
  }
}

module Panel() {
  size = [140, 180];
  radius = 5;
  r2 = radius * 2;

  hull() {
    ArrangeCentres2D([size[0] - r2, size[1] - r2]) {
      circle(r = radius, $fn = 24);
    }
  }
}

module MountingHoles() {
  ArrangeCentres2D([118.5, 161.5]) {
    circle(d = 4, $fn = 24);
  }
}

module PowerSocket() {
  square([51.75, 31.75], center = true);

  dx = 58.5 / 2;
  for(x = [-dx, dx]) {
    translate([x, 0]) {
      circle(d = 3.5, $fn = 24);
    }
  }
}

module AuxDigitalInputsSocket() {
  translate([0, -1.5]) {
    square([32.8, 13], center = true);
  }

  dx = 40 / 2;
  for(x = [-dx, dx]) {
    translate([x, 4.5]) {
      circle(d = 3.5, $fn = 24);
    }
  }
}

difference() {
  Panel();

  MountingHoles();

  translate([15, -50]) {
    PowerSocket();
  }

  translate([15, 40]) {
    AuxDigitalInputsSocket();
  }
}
