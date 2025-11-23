import { Component, inject } from '@angular/core';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { AuthService } from '@core/services/auth-service';
import { ThemeService } from '@core/services/theme-service';

@Component({
  selector: 'app-auth',
  imports: [ReactiveFormsModule],
  templateUrl: './auth.html',
  styleUrl: './auth.css',
})
export class Auth {
  authService = inject(AuthService);
  themeService = inject(ThemeService)
  isDarkMode = this.themeService.isDarkMode

  fb = new FormBuilder();
  form = this.fb.group({
    email: ['', [Validators.required, Validators.email]],
    password: ['', [Validators.required]],
  });

  handleSubmit() {
    this.submitted = true;
    if (this.form.valid) {
      const { email, password } = this.form.value;
      this.authService.authenticate(email!, password!).subscribe();
    }
  }

  onToggleTheme() {
    const isDark = this.themeService.isDarkMode();
    this.themeService.toggleDarkMode(!isDark);
  }

  submitted = false;

  isInvalid(controlName: string): boolean {
    const control = this.form.get(controlName);
    return !!(
      control &&
      control.invalid &&
      control.touched
    );
  }
}
