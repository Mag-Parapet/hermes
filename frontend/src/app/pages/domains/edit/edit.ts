import { AsyncPipe, CommonModule } from '@angular/common';
import { Component, inject, signal } from '@angular/core';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { UnitConvertPipe } from '@core/pipes/unit-convert-pipe';
import { DomainsService } from '@core/services/domains';
import { addParamHeader } from '@core/utils/add-param-header';
import { Store } from '@ngrx/store';
import { setLoading } from 'app/state/loading/loading.actions';
import Snackbar from 'awesome-snackbar';
import { catchError, tap } from 'rxjs';

@Component({
  selector: 'app-edit',
  imports: [ReactiveFormsModule, AsyncPipe, CommonModule, UnitConvertPipe],
  templateUrl: './edit.html',
  styleUrl: './edit.css',
})
export class Edit {
  domainsService = inject(DomainsService);
  router = inject(Router);
  route = inject(ActivatedRoute);
  store = inject(Store);

  domainId = this.route.snapshot.paramMap.get('id')!;

  domainTypes = [
    { value: 'reverse_proxy', label: 'Reverse Proxy' },
    { value: 'web_server', label: 'Web Server' },
    { value: 'static_host', label: 'Static Host' },
    { value: 'custom', label: 'Custom Config' },
  ];

  fb = new FormBuilder();
  form = this.fb.group({
    domain: ['', [Validators.required]],
    domainType: ['', [Validators.required]],
    
    nginxTargetHost: [''],
    nginxRootPath: [''],
    nginxConfigContent: [''],

    clientMaxBodySize: [1, [Validators.required, Validators.min(1)]],
    isSsl: [true],
    isActive: [true],
    sslCertificatePath: [''],
    sslCertificateKeyPath: [''],
  });

  isLoading = signal(true);

  domain$ = this.domainsService.getDomainById(this.domainId || '').pipe(
    tap((res: any) => {
      this.store.dispatch(setLoading({ state: false }))
      
      this.form.patchValue({
        domain: res.data.domain,
        domainType: res.data.domainType,
        clientMaxBodySize: res.data.clientMaxBodySize,
        isSsl: res.data.isSsl,
        isActive: res.data.isActive,
        
        nginxTargetHost: res.data.nginxTargetHost || '',
        nginxRootPath: res.data.nginxRootPath || '',
        nginxConfigContent: res.data.nginxConfigContent || '',
        
        sslCertificatePath: res.data.sslCertificatePath || '',
        sslCertificateKeyPath: res.data.sslCertificateKeyPath || ''
      });

      this.updateValidators(res.data.domainType);
      this.isLoading.set(false);
    }),
    addParamHeader(':domainId', 'data.domain'),
    catchError((err: any) => {
      if (err.status === 404) {
        this.router.navigate(['/404/']);
      }
      new Snackbar(err.error?.message || `Failed to load domain details`, {
        iconSrc: '/error.png',
        position: 'bottom-right',
      });
      this.router.navigate(['/domains/']);
      return [];
    })
  );
  
  constructor() {
    this.form.get('domainType')?.valueChanges.subscribe((type) => {
      this.updateValidators(type || '');
    });
  }

  updateValidators(type: string) {
    const targetHost = this.form.get('nginxTargetHost');
    const rootPath = this.form.get('nginxRootPath');
    const configContent = this.form.get('nginxConfigContent');
    const maxBody = this.form.get('clientMaxBodySize');
    const isSsl = this.form.get('isSsl');

    targetHost?.setValidators(null); targetHost?.disable();
    rootPath?.setValidators(null); rootPath?.disable();
    configContent?.setValidators(null); configContent?.disable();
    
    // Default enable common fields
    maxBody?.enable();
    isSsl?.enable();

    if (type === 'reverse_proxy') {
      targetHost?.enable();
      targetHost?.setValidators([Validators.required]);
    } else if (type === 'web_server' || type === 'static_host') {
      rootPath?.enable();
      rootPath?.setValidators([Validators.required]);
    } else if (type === 'custom') {
      configContent?.enable();
      configContent?.setValidators([Validators.required]);
      
      // Disable common fields for custom
      maxBody?.disable();
      isSsl?.disable();
      isSsl?.setValue(false);
    }
    
    targetHost?.updateValueAndValidity();
    rootPath?.updateValueAndValidity();
    configContent?.updateValueAndValidity();
  }

  onSubmit() {
    if (this.form.valid) {
      this.store.dispatch(setLoading({ state: true }));
      this.isLoading.set(true);
      
      const val = this.form.value;

      const payload: any = {
        domain: val.domain,
        domainType: val.domainType,
        clientMaxBodySize: val.clientMaxBodySize,
        isSsl: val.isSsl,
        isActive: val.isActive,
        
        nginxTargetHost: val.domainType === 'reverse_proxy' ? val.nginxTargetHost : undefined,
        nginxRootPath: ['web_server', 'static_host'].includes(val.domainType!) ? val.nginxRootPath : undefined,
        nginxConfigContent: val.domainType === 'custom' ? val.nginxConfigContent : undefined,
        
        sslCertificatePath: val.sslCertificatePath || undefined,
        sslCertificateKeyPath: val.sslCertificateKeyPath || undefined,
      };
      console.log(payload.isActive);
      this.domainsService.updateDomain(this.domainId, payload).subscribe(
        (res: any) => {
          this.isLoading.set(false);
          this.store.dispatch(setLoading({ state: false }));
          this.router.navigate(['/domains/', res.data.id]);
          new Snackbar('Domain updated successfully', {
            iconSrc: '/success.png',
            position: 'bottom-right',
          });
        },
        (err: any) => {
           this.isLoading.set(false);
           this.store.dispatch(setLoading({ state: false }));
           new Snackbar(err.error?.message || `Failed to update domain`, {
             iconSrc: '/error.png',
             position: 'bottom-right',
           });
           console.error(err);
        }
      );
    } else {
      this.form.markAllAsTouched();
    }
  }

  isInvalid(controlName: string): boolean {
    const control = this.form.get(controlName);
    return !!(
      control &&
      control.invalid &&
      control.touched &&
      control.enabled
    );
  }
}