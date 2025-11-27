import { JsonPipe } from '@angular/common';
import { Component, inject } from '@angular/core';
import { AuthService } from '@core/services/auth-service';

@Component({
  selector: 'app-home',
  imports: [JsonPipe],
  templateUrl: './home.html',
  styleUrl: './home.css',
})
export class Home {
  authService = inject(AuthService)

  currentUser = this.authService.currentUser;
}
