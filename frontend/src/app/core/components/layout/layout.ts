import { Component } from '@angular/core';
import { RouterOutlet } from '@angular/router';
import { Sidenav } from '../sidenav/sidenav';
import { Header } from '../header/header';

@Component({
  selector: 'app-layout',
  imports: [Sidenav, Header, RouterOutlet],
  templateUrl: './layout.html',
  styleUrl: './layout.css',
})
export class Layout {

}
