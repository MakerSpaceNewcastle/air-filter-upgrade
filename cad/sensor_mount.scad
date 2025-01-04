$fn = 32;

module ArrangeCentres(centres) {
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
  vertices = [
    [[-28, 35], 5],
    [[28, 35], 5],

    [[-50, -30], 20],
    [[50, -30], 20],
  ];

  hull() {
    for(v = vertices) {
      translate(v[0]) {
        circle(r = v[1]);
      }
    }
  }
}

module McuBoardMountingHoles() {
  ArrangeCentres(centres = [47, 23]) {
    circle(d = 3.1);
  }
}

module PmSensorMountingHoles() {
  ArrangeCentres(centres = [44.5, 33]) {
    circle(d = 2.1);
  }
}

module PirSensorMountingHoles() {
  centres_x = 20;

  dx = centres_x / 2;
  for(x = [-dx, dx]) {
    translate([x, 0]) {
      circle(d = 3.1);
    }
  }
}

difference() {
  Panel();

  for(x = [-25, 0, 25]) {
    translate([x, 35]) {
      circle(d = 4);
    }
  }

  translate([0, 12]) {
    McuBoardMountingHoles();
  }

  translate([0, -30]) {
    PmSensorMountingHoles();
  }

  translate([-50, -30]) {
    rotate([0, 0, -60]) {
      PirSensorMountingHoles();
    }
  }

  translate([50, -30]) {
    rotate([0, 0, 60]) {
      PirSensorMountingHoles();
    }
  }
}
